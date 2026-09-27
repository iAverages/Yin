use std::collections::HashMap;
use std::env;
use std::num::ParseIntError;
use std::time::Duration;

use posthog_rs::{Client, ClientOptionsBuilder, EvaluateFlagsOptions, FeatureFlagEvaluations};
use reqwest::{RequestBuilder, Response, StatusCode};
use serde::Deserialize;
use serde_json::{Value, json};

pub use posthog_rs::FlagValue;

const DEFAULT_HOST: &str = "https://eu.i.posthog.com";
const GUILD_GROUP: &str = "guild";
const CURRENT_GUILD_ID_PROPERTY: &str = "current_guild_id";
const MANAGEMENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeatureFlagScope {
    Global,
    User(u64),
    Guild(u64),
    Member { user_id: u64, guild_id: u64 },
}

impl FeatureFlagScope {
    fn properties(self) -> Vec<Value> {
        let property = |key: &str, value: u64| {
            json!({
                "key": key,
                "value": value.to_string(),
                "operator": "exact",
                "type": "person"
            })
        };
        match self {
            Self::Global => vec![],
            Self::User(user_id) => vec![property("distinct_id", user_id)],
            Self::Guild(guild_id) => vec![property(CURRENT_GUILD_ID_PROPERTY, guild_id)],
            Self::Member { user_id, guild_id } => vec![
                property("distinct_id", user_id),
                property(CURRENT_GUILD_ID_PROPERTY, guild_id),
            ],
        }
    }
}

#[derive(Default)]
pub struct FeatureFlags {
    evaluator: Option<Client>,
    management: Option<ManagementClient>,
}

impl FeatureFlags {
    pub async fn from_env() -> Result<Self, Error> {
        let host = optional_env("POSTHOG_HOST").unwrap_or_else(|| DEFAULT_HOST.to_owned());
        let app_host = optional_env("POSTHOG_APP_HOST").unwrap_or_else(|| management_host(&host));
        let evaluator = if let Some(project_token) = optional_env("POSTHOG_PROJECT_TOKEN") {
            let options = ClientOptionsBuilder::default()
                .api_key(project_token)
                .host(host)
                .build()
                .map_err(|error| Error::Configuration(error.to_string()))?;
            Some(posthog_rs::client(options).await)
        } else {
            None
        };
        let management = match (
            optional_env("POSTHOG_PROJECT_ID"),
            optional_env("POSTHOG_PERSONAL_API_KEY"),
        ) {
            (Some(project_id), Some(personal_api_key)) => Some(ManagementClient {
                http: reqwest::Client::new(),
                host: app_host.trim_end_matches('/').to_owned(),
                project_id: project_id.parse().map_err(Error::InvalidProjectId)?,
                personal_api_key,
            }),
            _ => None,
        };

        Ok(Self {
            evaluator,
            management,
        })
    }

    /// Evaluates a flag for a Discord user, optionally in a guild.
    ///
    /// PostHog receives the user ID as `distinct_id`, the guild ID as the
    /// `guild` group, and `current_guild_id` as a request-scoped person
    /// property. A person condition can combine `distinct_id` and
    /// `current_guild_id` for an exact user-in-guild override without storing
    /// Discord memberships in PostHog. Percentage and multivariate rollouts
    /// should target the `guild` group so every user in a guild stays together.
    pub async fn is_enabled(
        &self,
        key: &str,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> Result<bool, Error> {
        Ok(self
            .evaluate(key, user_id, guild_id)
            .await?
            .is_some_and(|flags| flags.is_enabled(key)))
    }

    /// Returns a boolean or multivariate flag value for a Discord user.
    pub async fn value(
        &self,
        key: &str,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> Result<Option<FlagValue>, Error> {
        Ok(self
            .evaluate(key, user_id, guild_id)
            .await?
            .and_then(|flags| flags.get_flag(key)))
    }

    /// Registers a Discord guild as PostHog's `guild` group and updates its name.
    pub fn identify_guild(&self, guild_id: u64, name: &str) -> Result<(), Error> {
        let Some(evaluator) = &self.evaluator else {
            return Ok(());
        };
        evaluator
            .group_identify(
                GUILD_GROUP,
                guild_id.to_string(),
                HashMap::from([("name", name)]),
            )
            .map_err(Error::Sdk)
    }

    /// Sets a boolean or multivariate flag value for one Discord scope.
    pub async fn set_value(
        &self,
        key: &str,
        scope: FeatureFlagScope,
        value: &str,
    ) -> Result<(), Error> {
        self.management
            .as_ref()
            .ok_or(Error::ManagementNotConfigured)?
            .set_value(key, scope, value)
            .await
    }

    async fn evaluate(
        &self,
        key: &str,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> Result<Option<FeatureFlagEvaluations>, Error> {
        let Some(evaluator) = &self.evaluator else {
            return Ok(None);
        };
        evaluator
            .evaluate_flags(user_id.to_string(), evaluation_options(key, guild_id))
            .await
            .map(Some)
            .map_err(Error::Sdk)
    }
}

struct ManagementClient {
    http: reqwest::Client,
    host: String,
    project_id: u64,
    personal_api_key: String,
}

impl ManagementClient {
    async fn set_value(
        &self,
        key: &str,
        scope: FeatureFlagScope,
        value: &str,
    ) -> Result<(), Error> {
        let flag = self.find_flag(key).await?;
        let value = flag_value(&flag.filters, value)?;
        let request = self
            .http
            .patch(format!(
                "{}/api/projects/{}/feature_flags/{}/",
                self.host, self.project_id, flag.id
            ))
            .bearer_auth(&self.personal_api_key)
            .json(&json!({
                "active": true,
                "filters": scoped_filters(flag.filters, scope, value)
            }));
        send(request).await?;
        Ok(())
    }

    async fn find_flag(&self, key: &str) -> Result<FeatureFlag, Error> {
        let request = self
            .http
            .get(format!(
                "{}/api/projects/{}/feature_flags/",
                self.host, self.project_id
            ))
            .bearer_auth(&self.personal_api_key)
            .query(&[("key", key)]);
        let page: FeatureFlagPage = send(request).await?.json().await?;
        page.results
            .into_iter()
            .find(|flag| flag.key == key)
            .ok_or_else(|| Error::FlagNotFound(key.to_owned()))
    }
}

fn flag_value(filters: &Value, value: &str) -> Result<FlagValue, Error> {
    if let Some(variants) = filters
        .pointer("/multivariate/variants")
        .and_then(Value::as_array)
    {
        if variants
            .iter()
            .any(|variant| variant.get("key").and_then(Value::as_str) == Some(value))
        {
            return Ok(FlagValue::String(value.to_owned()));
        }
        return Err(Error::InvalidValue {
            value: value.to_owned(),
            expected: variants
                .iter()
                .filter_map(|variant| variant.get("key").and_then(Value::as_str))
                .collect::<Vec<_>>()
                .join(", "),
        });
    }

    match value {
        "true" => Ok(FlagValue::Boolean(true)),
        "false" => Ok(FlagValue::Boolean(false)),
        _ => Err(Error::InvalidValue {
            value: value.to_owned(),
            expected: "true or false".to_owned(),
        }),
    }
}

fn scoped_filters(mut filters: Value, scope: FeatureFlagScope, value: FlagValue) -> Value {
    if !filters.is_object() {
        filters = json!({});
    }
    let filters = filters.as_object_mut().expect("filters is an object");
    let groups = filters.entry("groups").or_insert_with(|| json!([]));
    if !groups.is_array() {
        *groups = json!([]);
    }
    let groups = groups.as_array_mut().expect("groups is an array");
    let properties = scope.properties();

    if let Some(condition) = groups
        .iter_mut()
        .find(|condition| condition_has_properties(condition, &properties))
    {
        set_condition_value(condition, &value);
    } else {
        let mut condition = json!({
            "properties": properties,
        });
        set_condition_value(&mut condition, &value);
        groups.push(condition);
    }
    groups.sort_by_key(condition_rank);
    filters.insert("early_exit".to_owned(), true.into());

    Value::Object(filters.clone())
}

fn set_condition_value(condition: &mut Value, value: &FlagValue) {
    let condition = condition
        .as_object_mut()
        .expect("matching condition is an object");
    match value {
        FlagValue::Boolean(enabled) => {
            condition.insert(
                "rollout_percentage".to_owned(),
                if *enabled { 100 } else { 0 }.into(),
            );
            condition.remove("variant");
        }
        FlagValue::String(variant) => {
            condition.insert("rollout_percentage".to_owned(), 100.into());
            condition.insert("variant".to_owned(), variant.clone().into());
        }
    }
}

fn condition_has_properties(condition: &Value, expected: &[Value]) -> bool {
    let Some(properties) = condition.get("properties").and_then(Value::as_array) else {
        return expected.is_empty();
    };
    properties.len() == expected.len()
        && expected
            .iter()
            .all(|property| properties.contains(property))
}

fn condition_rank(condition: &Value) -> u8 {
    let Some(properties) = condition.get("properties").and_then(Value::as_array) else {
        return 4;
    };
    if properties.is_empty() {
        return 4;
    }
    let user = properties
        .iter()
        .any(|property| managed_property_key(property).is_some_and(|key| key == "distinct_id"));
    let guild = properties.iter().any(|property| {
        managed_property_key(property).is_some_and(|key| key == CURRENT_GUILD_ID_PROPERTY)
    });
    if properties
        .iter()
        .any(|property| managed_property_key(property).is_none())
    {
        return 3;
    }
    match (user, guild, properties.len()) {
        (true, true, 2) => 0,
        (true, false, 1) => 1,
        (false, true, 1) => 2,
        _ => 3,
    }
}

fn managed_property_key(property: &Value) -> Option<&str> {
    (property.get("type")? == "person" && property.get("operator")? == "exact")
        .then(|| property.get("key")?.as_str())?
}

async fn send(request: RequestBuilder) -> Result<Response, Error> {
    let response = request.timeout(MANAGEMENT_TIMEOUT).send().await?;
    let status = response.status();
    if status.is_success() {
        return Ok(response);
    }

    let body = response.text().await.unwrap_or_default();
    Err(Error::Management { status, body })
}

fn evaluation_options(key: &str, guild_id: Option<u64>) -> EvaluateFlagsOptions {
    let guild_id = guild_id.map(|guild_id| guild_id.to_string());
    EvaluateFlagsOptions {
        groups: guild_id
            .as_ref()
            .map(|guild_id| HashMap::from([(GUILD_GROUP.to_owned(), guild_id.clone())])),
        person_properties: guild_id.map(|guild_id| {
            HashMap::from([(CURRENT_GUILD_ID_PROPERTY.to_owned(), guild_id.into())])
        }),
        flag_keys: Some(vec![key.to_owned()]),
        ..Default::default()
    }
}

fn management_host(host: &str) -> String {
    host.trim_end_matches('/')
        .replace(".i.posthog.com", ".posthog.com")
}

fn optional_env(name: &str) -> Option<String> {
    env::var(name)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

#[derive(Deserialize)]
struct FeatureFlagPage {
    results: Vec<FeatureFlag>,
}

#[derive(Deserialize)]
struct FeatureFlag {
    id: u64,
    key: String,
    #[serde(default)]
    filters: Value,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("PostHog management is not configured")]
    ManagementNotConfigured,
    #[error("POSTHOG_PROJECT_ID must be an unsigned integer")]
    InvalidProjectId(#[source] ParseIntError),
    #[error("invalid PostHog configuration: {0}")]
    Configuration(String),
    #[error("invalid feature flag value {value:?}; expected {expected}")]
    InvalidValue { value: String, expected: String },
    #[error("PostHog SDK failed: {0}")]
    Sdk(#[source] posthog_rs::Error),
    #[error("PostHog request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("PostHog feature flag {0:?} was not found")]
    FlagNotFound(String),
    #[error("PostHog management API returned {status}: {body}")]
    Management { status: StatusCode, body: String },
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{Error, FeatureFlagScope, FeatureFlags, FlagValue, flag_value, scoped_filters};

    #[tokio::test]
    async fn disabled_when_not_configured() {
        let flags = FeatureFlags::default();

        assert!(!flags.is_enabled("test", 1, None).await.unwrap());
        assert!(flags.value("test", 1, None).await.unwrap().is_none());
        flags.identify_guild(1, "Test").unwrap();
        assert!(matches!(
            flags
                .set_value("test", FeatureFlagScope::Global, "true")
                .await,
            Err(Error::ManagementNotConfigured)
        ));
    }

    #[test]
    fn builds_and_orders_scoped_conditions() {
        let filters = json!({
            "groups": [{
                "properties": [{"key": "plan", "value": "pro", "operator": "exact", "type": "person"}],
                "rollout_percentage": 50
            }],
            "multivariate": {"variants": []}
        });
        let filters = scoped_filters(filters, FeatureFlagScope::Global, FlagValue::Boolean(false));
        let filters = scoped_filters(
            filters,
            FeatureFlagScope::Guild(20),
            FlagValue::Boolean(true),
        );
        let filters = scoped_filters(
            filters,
            FeatureFlagScope::User(10),
            FlagValue::Boolean(false),
        );
        let filters = scoped_filters(
            filters,
            FeatureFlagScope::Member {
                user_id: 10,
                guild_id: 20,
            },
            FlagValue::String("abembed".to_owned()),
        );

        assert_eq!(filters["early_exit"], true);
        assert_eq!(filters["multivariate"], json!({"variants": []}));
        assert_eq!(
            filters["groups"],
            json!([
                {
                    "properties": [
                        {"key": "distinct_id", "value": "10", "operator": "exact", "type": "person"},
                        {"key": "current_guild_id", "value": "20", "operator": "exact", "type": "person"}
                    ],
                    "rollout_percentage": 100,
                    "variant": "abembed"
                },
                {
                    "properties": [
                        {"key": "distinct_id", "value": "10", "operator": "exact", "type": "person"}
                    ],
                    "rollout_percentage": 0
                },
                {
                    "properties": [
                        {"key": "current_guild_id", "value": "20", "operator": "exact", "type": "person"}
                    ],
                    "rollout_percentage": 100
                },
                {
                    "properties": [
                        {"key": "plan", "value": "pro", "operator": "exact", "type": "person"}
                    ],
                    "rollout_percentage": 50
                },
                {"properties": [], "rollout_percentage": 0}
            ])
        );
    }

    #[test]
    fn parses_values_for_flag_type() {
        let boolean = json!({"groups": []});
        let multivariate = json!({
            "multivariate": {
                "variants": [
                    {"key": "fxtwitter", "rollout_percentage": 50},
                    {"key": "abembed", "rollout_percentage": 50}
                ]
            }
        });

        assert_eq!(
            flag_value(&boolean, "true").unwrap(),
            FlagValue::Boolean(true)
        );
        assert_eq!(
            flag_value(&multivariate, "abembed").unwrap(),
            FlagValue::String("abembed".to_owned())
        );
        assert!(matches!(
            flag_value(&multivariate, "missing"),
            Err(Error::InvalidValue { .. })
        ));
    }
}
