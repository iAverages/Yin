use std::collections::HashMap;
use std::env;
use std::num::ParseIntError;
use std::time::Duration;

use posthog_rs::{Client, ClientOptionsBuilder, EvaluateFlagsOptions, FeatureFlagEvaluations};
use reqwest::{RequestBuilder, Response, StatusCode};
use serde::Deserialize;

pub use posthog_rs::FlagValue;

const DEFAULT_HOST: &str = "https://eu.i.posthog.com";
const GUILD_GROUP: &str = "guild";
const CURRENT_GUILD_ID_PROPERTY: &str = "current_guild_id";
const MANAGEMENT_TIMEOUT: Duration = Duration::from_secs(10);

pub struct FeatureFlags {
    evaluator: Client,
    management: ManagementClient,
}

impl FeatureFlags {
    pub async fn from_env() -> Result<Self, Error> {
        let project_token = required_env("POSTHOG_PROJECT_TOKEN")?;
        let host = optional_env("POSTHOG_HOST").unwrap_or_else(|| DEFAULT_HOST.to_owned());
        let app_host = optional_env("POSTHOG_APP_HOST").unwrap_or_else(|| management_host(&host));
        let options = ClientOptionsBuilder::default()
            .api_key(project_token)
            .host(host.clone())
            .build()
            .map_err(|error| Error::Configuration(error.to_string()))?;

        Ok(Self {
            evaluator: posthog_rs::client(options).await,
            management: ManagementClient {
                http: reqwest::Client::new(),
                host: app_host.trim_end_matches('/').to_owned(),
                project_id: required_env("POSTHOG_PROJECT_ID")?
                    .parse()
                    .map_err(Error::InvalidProjectId)?,
                personal_api_key: required_env("POSTHOG_PERSONAL_API_KEY")?,
            },
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
        Ok(self.evaluate(key, user_id, guild_id).await?.is_enabled(key))
    }

    /// Returns a boolean or multivariate flag value for a Discord user.
    pub async fn value(
        &self,
        key: &str,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> Result<Option<FlagValue>, Error> {
        Ok(self.evaluate(key, user_id, guild_id).await?.get_flag(key))
    }

    /// Registers a Discord guild as PostHog's `guild` group and updates its name.
    pub fn identify_guild(&self, guild_id: u64, name: &str) -> Result<(), Error> {
        self.evaluator
            .group_identify(
                GUILD_GROUP,
                guild_id.to_string(),
                HashMap::from([("name", name)]),
            )
            .map_err(Error::Sdk)
    }

    /// Globally enables or disables a flag without changing its targeting rules.
    pub async fn set_enabled(&self, key: &str, enabled: bool) -> Result<(), Error> {
        self.management.set_enabled(key, enabled).await
    }

    async fn evaluate(
        &self,
        key: &str,
        user_id: u64,
        guild_id: Option<u64>,
    ) -> Result<FeatureFlagEvaluations, Error> {
        self.evaluator
            .evaluate_flags(user_id.to_string(), evaluation_options(key, guild_id))
            .await
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
    async fn set_enabled(&self, key: &str, enabled: bool) -> Result<(), Error> {
        let flag_id = self.find_flag_id(key).await?;
        let action = if enabled { "enable" } else { "disable" };
        let request = self
            .http
            .post(format!(
                "{}/api/projects/{}/feature_flags/{flag_id}/{action}/",
                self.host, self.project_id
            ))
            .bearer_auth(&self.personal_api_key);
        send(request).await?;
        Ok(())
    }

    async fn find_flag_id(&self, key: &str) -> Result<u64, Error> {
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
            .map(|flag| flag.id)
            .ok_or_else(|| Error::FlagNotFound(key.to_owned()))
    }
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

fn required_env(name: &'static str) -> Result<String, Error> {
    optional_env(name).ok_or(Error::MissingEnvironment(name))
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
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0} is required to configure PostHog feature flags")]
    MissingEnvironment(&'static str),
    #[error("POSTHOG_PROJECT_ID must be an unsigned integer")]
    InvalidProjectId(#[source] ParseIntError),
    #[error("invalid PostHog configuration: {0}")]
    Configuration(String),
    #[error("PostHog SDK failed: {0}")]
    Sdk(#[source] posthog_rs::Error),
    #[error("PostHog request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("PostHog feature flag {0:?} was not found")]
    FlagNotFound(String),
    #[error("PostHog management API returned {status}: {body}")]
    Management { status: StatusCode, body: String },
}
