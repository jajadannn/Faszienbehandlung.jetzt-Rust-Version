use chrono::Duration;
use serde::Deserialize;
use sqlx::SqlitePool;

use crate::{
    config::AppConfig,
    error::{AppError, AppResult},
    utils::{normalize_city, now_utc},
};

pub struct LocationService {
    client: reqwest::Client,
}

#[derive(Debug, Clone)]
pub struct LocationValidationResult {
    pub display_name: String,
}

#[derive(Debug, Deserialize)]
struct NominatimResult {
    display_name: String,
}

impl LocationService {
    pub fn from_config(config: &AppConfig) -> AppResult<Self> {
        let client = reqwest::Client::builder()
            .user_agent(config.geocoding_user_agent.clone())
            .timeout(std::time::Duration::from_secs(8))
            .build()?;

        Ok(Self { client })
    }

    pub async fn validate_city(
        &self,
        pool: &SqlitePool,
        city: &str,
    ) -> AppResult<LocationValidationResult> {
        let normalized = normalize_city(city);
        let now = now_utc();

        if let Some(row) = sqlx::query_as::<_, crate::models::LocationValidationCache>(
            r#"
            SELECT *
            FROM locations_validation_cache
            WHERE normalized_query = ?
              AND expires_at > ?
            ORDER BY created_at DESC
            LIMIT 1
            "#,
        )
        .bind(normalized.to_lowercase())
        .bind(now)
        .fetch_optional(pool)
        .await?
        {
            if row.is_valid {
                return Ok(LocationValidationResult {
                    display_name: row.display_name,
                });
            }
        }

        let response = self
            .client
            .get("https://nominatim.openstreetmap.org/search")
            .query(&[
                ("q", normalized.as_str()),
                ("format", "jsonv2"),
                ("limit", "1"),
                ("addressdetails", "1"),
            ])
            .send()
            .await
            .map_err(|_| {
                AppError::BadRequest(
                    "Die Ortsprüfung ist momentan nicht erreichbar. Bitte versuchen Sie es später erneut."
                        .to_string(),
                )
            })?;

        let results = response
            .json::<Vec<NominatimResult>>()
            .await
            .map_err(|_| {
                AppError::BadRequest(
                    "Die Ortsprüfung konnte nicht ausgewertet werden. Bitte versuchen Sie es später erneut."
                        .to_string(),
                )
            })?;

        let expires_at = now + Duration::days(30);

        if let Some(result) = results.first() {
            sqlx::query(
                r#"
                INSERT INTO locations_validation_cache
                    (query, normalized_query, display_name, is_valid, created_at, expires_at)
                VALUES (?, ?, ?, 1, ?, ?)
                "#,
            )
            .bind(city)
            .bind(normalized.to_lowercase())
            .bind(&result.display_name)
            .bind(now)
            .bind(expires_at)
            .execute(pool)
            .await?;

            return Ok(LocationValidationResult {
                display_name: result.display_name.clone(),
            });
        }

        sqlx::query(
            r#"
            INSERT INTO locations_validation_cache
                (query, normalized_query, display_name, is_valid, created_at, expires_at)
            VALUES (?, ?, ?, 0, ?, ?)
            "#,
        )
        .bind(city)
        .bind(normalized.to_lowercase())
        .bind(city)
        .bind(now)
        .bind(expires_at)
        .execute(pool)
        .await?;

        Err(AppError::BadRequest(
            "Bitte geben Sie einen real existierenden Wohnort an.".to_string(),
        ))
    }
}
