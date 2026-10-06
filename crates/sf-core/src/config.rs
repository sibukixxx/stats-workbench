use serde::{Deserialize, Serialize};

/// Application-level configuration loaded from config/default.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub defaults: DefaultsConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    #[serde(default = "default_output_format")]
    pub output_format: String,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            output_format: default_output_format(),
        }
    }
}

fn default_output_format() -> String {
    "terminal".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: default_log_level(),
        }
    }
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            defaults: DefaultsConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}
