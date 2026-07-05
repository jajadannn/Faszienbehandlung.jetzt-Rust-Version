use std::str::FromStr;

use axum::Router;
use faszienbehandlung_jetzt::{
    config::AppConfig,
    db::MIGRATOR,
    handlers,
    services::{email::EmailService, location::LocationService},
    state::AppState,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

pub async fn test_app() -> (Router, AppConfig) {
    let config = AppConfig::from_env().expect("Konfiguration mit Defaults muss parsen");

    let options = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("SQLite-Optionen")
        .foreign_keys(true);

    // Wichtig: genau 1 Verbindung – jede weitere :memory:-Verbindung
    // wäre eine separate, leere Datenbank.
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .expect("In-Memory-SQLite verbindet");

    MIGRATOR.run(&pool).await.expect("Migrationen laufen durch");

    let email_service = EmailService::from_config(&config).expect("EmailService (LogOnly)");
    let location_service = LocationService::from_config(&config).expect("LocationService");

    let state = AppState::new(config.clone(), pool, email_service, location_service);

    (handlers::router(state), config)
}
