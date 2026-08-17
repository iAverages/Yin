use std::env;
use std::net::SocketAddr;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub struct ApiConfig {
    pub bind_addr: SocketAddr,
    pub auth_service_url: String,
    pub cors_allowed_origins: Vec<String>,
}

impl ApiConfig {
    pub fn from_env() -> Result<Self, Error> {
        Ok(Self {
            bind_addr: bind_addr()?,
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://auth:3001".to_owned()),
            cors_allowed_origins: env::var("API_CORS_ALLOWED_ORIGINS")
                .unwrap_or_else(|_| "http://panel.yin.localhost,http://localhost:3000".to_owned())
                .split(',')
                .map(str::trim)
                .filter(|origin| !origin.is_empty())
                .map(str::to_owned)
                .collect(),
        })
    }
}

fn bind_addr() -> Result<SocketAddr, Error> {
    let value = env::var("API_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_owned());
    Ok(value.parse()?)
}
