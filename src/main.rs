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

    let config = AppConfig::from_env()?;
    let pool = db::init_pool(&config).await?;
    let email_service = EmailService::from_config(&config)?;
    let location_service = LocationService::from_config(&config)?;

    let state = AppState::new(config.clone(), pool, email_service, location_service);

    match std::env::args().nth(1).as_deref() {
        Some("seed-demo") => {
            seed::seed_demo(&state).await?;
            info!("Seed-Daten wurden erfolgreich eingespielt.");
            return Ok(());
        }
        Some("print-config") => {
            println!("BASE_URL={}", config.base_url);
            println!("PRACTICE_NAME={}", config.practice_name);
            println!("PRACTITIONER_NAME={}", config.practitioner_name);
            println!("PRACTICE_EMAIL={}", config.practice_email);
            println!("PRACTICE_PHONE={}", config.practice_phone);
            println!("PRACTICE_ADDRESS_LINE_1={}", config.practice_address_line_1);
            println!("PRACTICE_ADDRESS_LINE_2={}", config.practice_address_line_2);
            println!("PRACTICE_REGION_LABEL={}", config.practice_region_label);
            println!("PRACTICE_HOUSE_CALL_AREA={}", config.practice_house_call_area);
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
            return Ok(());
        }
        _ => {}
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
