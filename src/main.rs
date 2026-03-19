use std::net::SocketAddr;

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
    dotenvy::dotenv().ok();
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
            info!("Konfiguration geladen fuer {}", config.base_url);
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
