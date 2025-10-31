//! Configuration management

use crate::error::{DashboardError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Main dashboard configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardConfig {
    /// Server configuration
    pub server: ServerConfig,

    /// Database configuration
    pub database: DatabaseConfig,

    /// Authentication configuration
    pub auth: AuthConfig,

    /// CORS configuration
    #[serde(default)]
    pub cors: CorsConfig,

    /// Logging configuration
    #[serde(default)]
    pub logging: LoggingConfig,
}

impl DashboardConfig {
    /// Load configuration from file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::from(path.as_ref()))
            .add_source(
                config::Environment::with_prefix("DASHBOARD")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .map_err(|e| DashboardError::Configuration(e.to_string()))?;

        let dashboard_config: Self = config
            .try_deserialize()
            .map_err(|e| DashboardError::Configuration(e.to_string()))?;

        dashboard_config.validate()?;
        Ok(dashboard_config)
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(
                config::Environment::with_prefix("DASHBOARD")
                    .separator("__")
                    .try_parsing(true),
            )
            .build()
            .map_err(|e| DashboardError::Configuration(e.to_string()))?;

        let dashboard_config: Self = config
            .try_deserialize()
            .map_err(|e| DashboardError::Configuration(e.to_string()))?;

        dashboard_config.validate()?;
        Ok(dashboard_config)
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        self.server.validate()?;
        self.database.validate()?;
        self.auth.validate()?;
        Ok(())
    }
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            database: DatabaseConfig::default(),
            auth: AuthConfig::default(),
            cors: CorsConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host
    #[serde(default = "default_host")]
    pub host: String,

    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,

    /// Maximum request body size in bytes
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
}

impl ServerConfig {
    /// Get bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Validate server configuration
    pub fn validate(&self) -> Result<()> {
        if self.port == 0 {
            return Err(DashboardError::Configuration(
                "Port cannot be 0".to_string(),
            ));
        }

        if self.timeout_secs == 0 {
            return Err(DashboardError::Configuration(
                "Timeout must be greater than 0".to_string(),
            ));
        }

        if self.max_body_size == 0 {
            return Err(DashboardError::Configuration(
                "Max body size must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            timeout_secs: default_timeout(),
            max_body_size: default_max_body_size(),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,

    /// Maximum number of connections
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Minimum number of connections
    #[serde(default = "default_min_connections")]
    pub min_connections: u32,

    /// Connection timeout in seconds
    #[serde(default = "default_connection_timeout")]
    pub connection_timeout_secs: u64,

    /// Enable SQL logging
    #[serde(default)]
    pub enable_logging: bool,
}

impl DatabaseConfig {
    /// Validate database configuration
    pub fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err(DashboardError::Configuration(
                "Database URL cannot be empty".to_string(),
            ));
        }

        if self.max_connections == 0 {
            return Err(DashboardError::Configuration(
                "Max connections must be greater than 0".to_string(),
            ));
        }

        if self.min_connections > self.max_connections {
            return Err(DashboardError::Configuration(
                "Min connections cannot exceed max connections".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "postgres://localhost/llm_shield_dashboard".to_string(),
            max_connections: default_max_connections(),
            min_connections: default_min_connections(),
            connection_timeout_secs: default_connection_timeout(),
            enable_logging: false,
        }
    }
}

/// Authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT secret key (base64 encoded)
    pub jwt_secret: String,

    /// JWT expiration time in seconds
    #[serde(default = "default_jwt_expiration")]
    pub jwt_expiration_secs: u64,

    /// Enable API key authentication
    #[serde(default = "default_enable_api_keys")]
    pub enable_api_keys: bool,

    /// API key header name
    #[serde(default = "default_api_key_header")]
    pub api_key_header: String,
}

impl AuthConfig {
    /// Validate auth configuration
    pub fn validate(&self) -> Result<()> {
        if self.jwt_secret.is_empty() {
            return Err(DashboardError::Configuration(
                "JWT secret cannot be empty".to_string(),
            ));
        }

        if self.jwt_expiration_secs == 0 {
            return Err(DashboardError::Configuration(
                "JWT expiration must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_secret: "CHANGE_ME_IN_PRODUCTION".to_string(),
            jwt_expiration_secs: default_jwt_expiration(),
            enable_api_keys: default_enable_api_keys(),
            api_key_header: default_api_key_header(),
        }
    }
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Allowed origins
    #[serde(default = "default_allowed_origins")]
    pub allowed_origins: Vec<String>,

    /// Allowed methods
    #[serde(default = "default_allowed_methods")]
    pub allowed_methods: Vec<String>,

    /// Allowed headers
    #[serde(default = "default_allowed_headers")]
    pub allowed_headers: Vec<String>,

    /// Allow credentials
    #[serde(default)]
    pub allow_credentials: bool,

    /// Max age in seconds
    #[serde(default = "default_max_age")]
    pub max_age_secs: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: default_allowed_origins(),
            allowed_methods: default_allowed_methods(),
            allowed_headers: default_allowed_headers(),
            allow_credentials: false,
            max_age_secs: default_max_age(),
        }
    }
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Log format (json or pretty)
    #[serde(default = "default_log_format")]
    pub format: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
            format: default_log_format(),
        }
    }
}

// Default value functions
fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    4000
}

fn default_timeout() -> u64 {
    30
}

fn default_max_body_size() -> usize {
    10 * 1024 * 1024 // 10 MB
}

fn default_max_connections() -> u32 {
    20
}

fn default_min_connections() -> u32 {
    5
}

fn default_connection_timeout() -> u64 {
    30
}

fn default_jwt_expiration() -> u64 {
    900 // 15 minutes
}

fn default_enable_api_keys() -> bool {
    true
}

fn default_api_key_header() -> String {
    "X-API-Key".to_string()
}

fn default_allowed_origins() -> Vec<String> {
    vec!["http://localhost:3000".to_string()]
}

fn default_allowed_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
        "OPTIONS".to_string(),
    ]
}

fn default_allowed_headers() -> Vec<String> {
    vec![
        "Content-Type".to_string(),
        "Authorization".to_string(),
        "X-API-Key".to_string(),
    ]
}

fn default_max_age() -> u64 {
    3600
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_log_format() -> String {
    "pretty".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DashboardConfig::default();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 4000);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_server_config_bind_address() {
        let config = ServerConfig {
            host: "0.0.0.0".to_string(),
            port: 8080,
            ..Default::default()
        };
        assert_eq!(config.bind_address(), "0.0.0.0:8080");
    }

    #[test]
    fn test_server_config_validation() {
        let mut config = ServerConfig::default();

        // Valid config
        assert!(config.validate().is_ok());

        // Invalid port
        config.port = 0;
        assert!(config.validate().is_err());

        // Valid port
        config.port = 8080;
        assert!(config.validate().is_ok());

        // Invalid timeout
        config.timeout_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_database_config_validation() {
        let mut config = DatabaseConfig::default();

        // Valid config
        assert!(config.validate().is_ok());

        // Empty URL
        config.url = String::new();
        assert!(config.validate().is_err());

        // Min > Max connections
        config.url = "postgres://localhost".to_string();
        config.min_connections = 10;
        config.max_connections = 5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_auth_config_validation() {
        let mut config = AuthConfig::default();

        // Valid config (even with default "CHANGE_ME" secret for testing)
        assert!(config.validate().is_ok());

        // Empty JWT secret
        config.jwt_secret = String::new();
        assert!(config.validate().is_err());

        // Zero expiration
        config.jwt_secret = "test_secret".to_string();
        config.jwt_expiration_secs = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::default();
        assert_eq!(config.allowed_origins, vec!["http://localhost:3000"]);
        assert!(config.allowed_methods.contains(&"GET".to_string()));
        assert!(config.allowed_methods.contains(&"POST".to_string()));
    }

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert_eq!(config.format, "pretty");
    }
}
