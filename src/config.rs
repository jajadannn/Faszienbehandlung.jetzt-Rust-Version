use std::env;

use anyhow::Context;

use crate::error::AppResult;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub app_env: String,
    pub bind_address: String,
    pub base_url: String,
    pub database_url: String,
    pub session_cookie_secure: bool,
    pub session_ttl_hours: i64,
    pub practice_name: String,
    pub practice_email: String,
    pub practice_phone: String,
    pub practice_address_line_1: String,
    pub practice_address_line_2: String,
    pub booking_base_price_cents: i64,
    pub geocoding_user_agent: String,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_username: Option<String>,
    pub smtp_password: Option<String>,
    pub smtp_from: String,
}

impl AppConfig {
    pub fn from_env() -> AppResult<Self> {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let bind_address =
            env::var("BIND_ADDRESS").unwrap_or_else(|_| "127.0.0.1:3000".to_string());
        let base_url = env::var("BASE_URL")
            .unwrap_or_else(|_| "https://www.faszienbehandlung.jetzt".to_string());
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite://data/faszienbehandlung.db".to_string());
        let session_cookie_secure = env::var("SESSION_COOKIE_SECURE")
            .unwrap_or_else(|_| {
                if app_env == "production" {
                    "true"
                } else {
                    "false"
                }
                .to_string()
            })
            .parse()
            .context("SESSION_COOKIE_SECURE muss true oder false sein")?;
        let session_ttl_hours = env::var("SESSION_TTL_HOURS")
            .unwrap_or_else(|_| "168".to_string())
            .parse()
            .context("SESSION_TTL_HOURS muss eine Zahl sein")?;
        let practice_name = env::var("PRACTICE_NAME")
            .unwrap_or_else(|_| "Praxis fuer Faszienbehandlung Jetzt".to_string());
        let practice_email = env::var("PRACTICE_EMAIL")
            .unwrap_or_else(|_| "kontakt@faszienbehandlung.jetzt".to_string());
        let practice_phone =
            env::var("PRACTICE_PHONE").unwrap_or_else(|_| "+49 30 0000 0000".to_string());
        let practice_address_line_1 =
            env::var("PRACTICE_ADDRESS_LINE_1").unwrap_or_else(|_| "Musterstrasse 12".to_string());
        let practice_address_line_2 =
            env::var("PRACTICE_ADDRESS_LINE_2").unwrap_or_else(|_| "10115 Berlin".to_string());
        let booking_base_price_cents = env::var("BOOKING_BASE_PRICE_CENTS")
            .unwrap_or_else(|_| "8900".to_string())
            .parse()
            .context("BOOKING_BASE_PRICE_CENTS muss eine Zahl sein")?;
        let geocoding_user_agent = env::var("GEOCODING_USER_AGENT").unwrap_or_else(|_| {
            "faszienbehandlung-jetzt/1.0 (kontakt@faszienbehandlung.jetzt)".to_string()
        });
        let smtp_host = env::var("SMTP_HOST")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let smtp_port = env::var("SMTP_PORT")
            .unwrap_or_else(|_| "587".to_string())
            .parse()
            .context("SMTP_PORT muss eine Zahl sein")?;
        let smtp_username = env::var("SMTP_USERNAME")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let smtp_password = env::var("SMTP_PASSWORD")
            .ok()
            .filter(|value| !value.trim().is_empty());
        let smtp_from =
            env::var("SMTP_FROM").unwrap_or_else(|_| "noreply@faszienbehandlung.jetzt".to_string());

        Ok(Self {
            app_env,
            bind_address,
            base_url,
            database_url,
            session_cookie_secure,
            session_ttl_hours,
            practice_name,
            practice_email,
            practice_phone,
            practice_address_line_1,
            practice_address_line_2,
            booking_base_price_cents,
            geocoding_user_agent,
            smtp_host,
            smtp_port,
            smtp_username,
            smtp_password,
            smtp_from,
        })
    }
}
