use std::sync::Arc;

use axum::extract::FromRef;
use sqlx::SqlitePool;

use crate::{
    config::AppConfig,
    services::{email::EmailService, location::LocationService},
};

#[derive(Clone, FromRef)]
pub struct AppState {
    pub config: Arc<AppConfig>,
    pub pool: SqlitePool,
    pub email_service: Arc<EmailService>,
    pub location_service: Arc<LocationService>,
}

impl AppState {
    pub fn new(
        config: AppConfig,
        pool: SqlitePool,
        email_service: EmailService,
        location_service: LocationService,
    ) -> Self {
        Self {
            config: Arc::new(config),
            pool,
            email_service: Arc::new(email_service),
            location_service: Arc::new(location_service),
        }
    }
}
