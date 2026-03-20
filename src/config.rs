use std::env;

use anyhow::{Context, anyhow};

use crate::error::AppResult;

#[derive(Clone, Debug)]
pub enum SmtpSecurity {
    Auto,
    ImplicitTls,
    StartTls,
    Plain,
}

impl SmtpSecurity {
    pub fn parse(value: &str) -> AppResult<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "implicit_tls" | "implicit-tls" | "ssl" | "tls" | "smtps" => Ok(Self::ImplicitTls),
            "starttls" | "start_tls" | "start-tls" => Ok(Self::StartTls),
            "plain" | "plaintext" | "none" => Ok(Self::Plain),
            other => Err(anyhow!(
                "SMTP_SECURITY muss auto, implicit_tls, starttls oder plain sein (aktuell: {other})"
            )
            .into()),
        }
    }

    pub fn resolved_for_port(&self, port: u16) -> Self {
        match self {
            Self::Auto => match port {
                465 => Self::ImplicitTls,
                587 => Self::StartTls,
                _ => Self::Plain,
            },
            other => other.clone(),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::ImplicitTls => "implicit_tls",
            Self::StartTls => "starttls",
            Self::Plain => "plain",
        }
    }
}

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub app_env: String,
    pub bind_address: String,
    pub base_url: String,
    pub database_url: String,
    pub auto_reload_enabled: bool,
    pub auto_reload_interval_ms: u64,
    pub session_cookie_secure: bool,
    pub session_ttl_hours: i64,
    pub practice_name: String,
    pub practitioner_name: String,
    pub practice_email: String,
    pub practice_phone: String,
    pub practice_address_line_1: String,
    pub practice_address_line_2: String,
    pub practice_region_label: String,
    pub practice_house_call_area: String,
    pub opening_hours_weekdays: String,
    pub opening_hours_saturday: String,
    pub appointment_duration_minutes: i64,
    pub booking_base_price_cents: i64,
    pub booking_package_session_price_cents: i64,
    pub booking_package_session_count: i64,
    pub booking_package_validity_months: i64,
    pub house_call_fee_cents: i64,
    pub email_resend_cooldown_seconds: i64,
    pub geocoding_user_agent: String,
    pub smtp_host: Option<String>,
    pub smtp_port: u16,
    pub smtp_security: SmtpSecurity,
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
        let auto_reload_enabled = env::var("AUTO_RELOAD_ENABLED")
            .unwrap_or_else(|_| {
                if app_env == "production" {
                    "false"
                } else {
                    "true"
                }
                .to_string()
            })
            .parse()
            .context("AUTO_RELOAD_ENABLED muss true oder false sein")?;
        let auto_reload_interval_ms = env::var("AUTO_RELOAD_INTERVAL_MS")
            .unwrap_or_else(|_| "1200".to_string())
            .parse()
            .context("AUTO_RELOAD_INTERVAL_MS muss eine Zahl sein")?;
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
            .unwrap_or_else(|_| "Praxis für Faszienbehandlung Jetzt".to_string());
        let practitioner_name =
            env::var("PRACTITIONER_NAME").unwrap_or_else(|_| practice_name.clone());
        let practice_email = env::var("PRACTICE_EMAIL")
            .unwrap_or_else(|_| "kontakt@faszienbehandlung.jetzt".to_string());
        let practice_phone =
            env::var("PRACTICE_PHONE").unwrap_or_else(|_| "+49 30 0000 0000".to_string());
        let practice_address_line_1 =
            env::var("PRACTICE_ADDRESS_LINE_1").unwrap_or_else(|_| "Musterstraße 12".to_string());
        let practice_address_line_2 =
            env::var("PRACTICE_ADDRESS_LINE_2").unwrap_or_else(|_| "10115 Berlin".to_string());
        let practice_region_label = env::var("PRACTICE_REGION_LABEL")
            .unwrap_or_else(|_| practice_address_line_2.clone());
        let practice_house_call_area = env::var("PRACTICE_HOUSE_CALL_AREA")
            .unwrap_or_else(|_| practice_region_label.clone());
        let opening_hours_weekdays = env::var("OPENING_HOURS_WEEKDAYS")
            .unwrap_or_else(|_| "Mo-Fr 16:00-22:00 Uhr".to_string());
        let opening_hours_saturday = env::var("OPENING_HOURS_SATURDAY")
            .unwrap_or_else(|_| "Sa 09:00-19:00 Uhr".to_string());
        let appointment_duration_minutes: i64 = env::var("APPOINTMENT_DURATION_MINUTES")
            .unwrap_or_else(|_| "90".to_string())
            .parse()
            .context("APPOINTMENT_DURATION_MINUTES muss eine Zahl sein")?;
        let booking_base_price_cents: i64 = env::var("BOOKING_BASE_PRICE_CENTS")
            .unwrap_or_else(|_| "8900".to_string())
            .parse()
            .context("BOOKING_BASE_PRICE_CENTS muss eine Zahl sein")?;
        let booking_package_session_price_cents: i64 =
            env::var("BOOKING_PACKAGE_SESSION_PRICE_CENTS")
            .unwrap_or_else(|_| booking_base_price_cents.to_string())
            .parse()
            .context("BOOKING_PACKAGE_SESSION_PRICE_CENTS muss eine Zahl sein")?;
        let booking_package_session_count: i64 = env::var("BOOKING_PACKAGE_SESSION_COUNT")
            .unwrap_or_else(|_| "10".to_string())
            .parse()
            .context("BOOKING_PACKAGE_SESSION_COUNT muss eine Zahl sein")?;
        let booking_package_validity_months: i64 = env::var("BOOKING_PACKAGE_VALIDITY_MONTHS")
            .unwrap_or_else(|_| "12".to_string())
            .parse()
            .context("BOOKING_PACKAGE_VALIDITY_MONTHS muss eine Zahl sein")?;
        let house_call_fee_cents: i64 = env::var("HOUSE_CALL_FEE_CENTS")
            .unwrap_or_else(|_| "1000".to_string())
            .parse()
            .context("HOUSE_CALL_FEE_CENTS muss eine Zahl sein")?;
        let email_resend_cooldown_seconds: i64 = env::var("EMAIL_RESEND_COOLDOWN_SECONDS")
            .unwrap_or_else(|_| "180".to_string())
            .parse()
            .context("EMAIL_RESEND_COOLDOWN_SECONDS muss eine Zahl sein")?;
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
        let smtp_security = env::var("SMTP_SECURITY")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(|value| SmtpSecurity::parse(&value))
            .transpose()?
            .unwrap_or(SmtpSecurity::Auto);
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
            auto_reload_enabled,
            auto_reload_interval_ms,
            session_cookie_secure,
            session_ttl_hours,
            practice_name,
            practitioner_name,
            practice_email,
            practice_phone,
            practice_address_line_1,
            practice_address_line_2,
            practice_region_label,
            practice_house_call_area,
            opening_hours_weekdays,
            opening_hours_saturday,
            appointment_duration_minutes,
            booking_base_price_cents,
            booking_package_session_price_cents,
            booking_package_session_count,
            booking_package_validity_months,
            house_call_fee_cents,
            email_resend_cooldown_seconds,
            geocoding_user_agent,
            smtp_host,
            smtp_port,
            smtp_security,
            smtp_username,
            smtp_password,
            smtp_from,
        })
    }
}
