use serde_json::Value;
use twilight_http::Client;
use twilight_http::request::Request;
use twilight_http::routing::Route;

use crate::Error;

pub async fn request(
    http: &Client,
    route: &Route<'_>,
    body: Option<&Value>,
) -> Result<Value, Error> {
    let mut request = Request::builder(route);

    if let Some(body) = body {
        request = request.json(body);
    }

    let bytes = http
        .request::<Value>(request.build()?)
        .await?
        .bytes()
        .await?;

    if bytes.is_empty() {
        return Ok(Value::Null);
    }

    Ok(serde_json::from_slice(&bytes)?)
}
