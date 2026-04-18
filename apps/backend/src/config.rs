use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub app_port: u16,
    pub redis_url: String,
    pub database_url: String,
    pub mock_mode: bool,
    pub log_level: String,
    pub timezone: String,
}

impl Config {
    pub fn from_env() -> Self {
        let app_port = env::var("APP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080);

        let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| {
            "redis://redis:6379".to_string()
        });

        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://council:council-dev-password@postgres:5432/council".to_string()
        });

        let mock_mode = env::var("MOCK_MODE")
            .ok()
            .map(|v| v.to_lowercase() == "true" || v == "1")
            .unwrap_or(false);

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| {
            "info".to_string()
        });

        let timezone = env::var("TIMEZONE").unwrap_or_else(|_| {
            env::var("TZ").unwrap_or_else(|_| "UTC".to_string())
        });

        Config {
            app_port,
            redis_url,
            database_url,
            mock_mode,
            log_level,
            timezone,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config {
            app_port: 8080,
            redis_url: "redis://localhost:6379".to_string(),
            database_url: "postgresql://localhost:5432/test".to_string(),
            mock_mode: false,
            log_level: "info".to_string(),
            timezone: "UTC".to_string(),
        };

        assert_eq!(config.app_port, 8080);
        assert!(!config.mock_mode);
    }
}
