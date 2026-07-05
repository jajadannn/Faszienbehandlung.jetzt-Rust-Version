use std::{net::SocketAddr, path::PathBuf};

use faszienbehandlung_jetzt::{
    config::AppConfig,
    db,
    error::AppResult,
    handlers, seed,
    services::{email::EmailService, location::LocationService},
    state::AppState,
};
use tokio::net::TcpListener;
use tracing::info;

#[tokio::main]
async fn main() -> AppResult<()> {
    load_env();
    init_tracing();

    let command = std::env::args().nth(1);
    let config = AppConfig::from_env()?;
    let email_service = EmailService::from_config(&config)?;

    match command.as_deref() {
        Some("print-config") => {
            println!("BASE_URL={}", config.base_url);
            println!("AUTO_RELOAD_ENABLED={}", config.auto_reload_enabled);
            println!("AUTO_RELOAD_INTERVAL_MS={}", config.auto_reload_interval_ms);
            println!("PRACTICE_NAME={}", config.practice_name);
            println!("PRACTITIONER_NAME={}", config.practitioner_name);
            println!("PRACTICE_EMAIL={}", config.practice_email);
            println!("PRACTICE_PHONE={}", config.practice_phone);
            println!("PRACTICE_ADDRESS_LINE_1={}", config.practice_address_line_1);
            println!("PRACTICE_ADDRESS_LINE_2={}", config.practice_address_line_2);
            println!("PRACTICE_REGION_LABEL={}", config.practice_region_label);
            println!(
                "PRACTICE_HOUSE_CALL_AREA={}",
                config.practice_house_call_area
            );
            println!("OPENING_HOURS_WEEKDAYS={}", config.opening_hours_weekdays);
            println!("OPENING_HOURS_SATURDAY={}", config.opening_hours_saturday);
            println!(
                "BOOKING_BASE_PRICE_CENTS={}",
                config.booking_base_price_cents
            );
            println!(
                "BOOKING_PACKAGE_SESSION_PRICE_CENTS={}",
                config.booking_package_session_price_cents
            );
            println!("HOUSE_CALL_FEE_CENTS={}", config.house_call_fee_cents);
            println!(
                "EMAIL_RESEND_COOLDOWN_SECONDS={}",
                config.email_resend_cooldown_seconds
            );
            println!("SMTP_HOST={}", config.smtp_host.as_deref().unwrap_or(""));
            println!("SMTP_PORT={}", config.smtp_port);
            println!("SMTP_SECURITY={}", config.smtp_security.as_str());
            println!("SMTP_FROM={}", config.smtp_from);
            println!("SMTP_USERNAME_SET={}", config.smtp_username.is_some());
            println!("SMTP_PASSWORD_SET={}", config.smtp_password.is_some());
            return Ok(());
        }
        Some("smtp-test") => {
            println!("SMTP_HOST={}", config.smtp_host.as_deref().unwrap_or(""));
            println!("SMTP_PORT={}", config.smtp_port);
            println!(
                "SMTP_SECURITY={}",
                config
                    .smtp_security
                    .resolved_for_port(config.smtp_port)
                    .as_str()
            );
            println!("SMTP_FROM={}", config.smtp_from);
            match email_service.test_connection().await? {
                Some(true) => println!("SMTP_CONNECTION=ok"),
                Some(false) => println!("SMTP_CONNECTION=failed"),
                None => println!("SMTP_CONNECTION=not_configured"),
            }
            return Ok(());
        }
        _ => {}
    }

    let pool = db::init_pool(&config).await?;
    let location_service = LocationService::from_config(&config)?;
    let state = AppState::new(config.clone(), pool, email_service, location_service);

    if matches!(command.as_deref(), Some("seed-demo")) {
        seed::seed_demo(&state).await?;
        info!("Seed-Daten wurden erfolgreich eingespielt.");
        return Ok(());
    }

    let app = handlers::router(state.clone());
    let addr: SocketAddr = config.bind_address.parse()?;
    let listener = TcpListener::bind(addr).await?;
    info!("Server lauscht auf http://{}", addr);
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() {
    let env_filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "faszienbehandlung_jetzt=info,tower_http=info,axum=info".into());

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .compact()
        .init();
}

fn load_env() {
    if dotenvy::dotenv().is_ok() {
        return;
    }

    let manifest_env = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".env");
    let _ = dotenvy::from_path(&manifest_env);
}
