use chrono::NaiveDateTime;
use serde::Deserialize;
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct User {
    pub id: i64,
    pub full_name: String,
    pub email: String,
    pub email_verified: bool,
    pub phone_number: String,
    pub city: String,
    pub password_hash: String,
    pub role: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct Customer {
    pub id: i64,
    pub user_id: i64,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct AuthenticatedUser {
    pub user_id: i64,
    pub customer_id: Option<i64>,
    pub full_name: String,
    pub email: String,
    pub email_verified: bool,
    pub phone_number: String,
    pub city: String,
    pub role: String,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, FromRow)]
pub struct Appointment {
    pub id: i64,
    pub customer_id: i64,
    pub desired_at: NaiveDateTime,
    pub status: String,
    pub message: Option<String>,
    pub total_amount_cents: i64,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct Payment {
    pub id: i64,
    pub customer_id: i64,
    pub appointment_id: Option<i64>,
    pub amount_total_cents: i64,
    pub amount_paid_cents: i64,
    pub amount_open_cents: i64,
    pub status: String,
    pub payment_date: Option<NaiveDateTime>,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct PaymentEvent {
    pub id: i64,
    pub payment_id: i64,
    pub recorded_by_user_id: Option<i64>,
    pub amount_cents: i64,
    pub note: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct AdminNote {
    pub id: i64,
    pub customer_id: i64,
    pub admin_user_id: i64,
    pub note: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub expires_at: NaiveDateTime,
    pub revoked_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub last_seen_at: NaiveDateTime,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct EmailVerification {
    pub id: i64,
    pub user_id: i64,
    pub token_hash: String,
    pub email: String,
    pub purpose: String,
    pub expires_at: NaiveDateTime,
    pub consumed_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
pub struct LocationValidationCache {
    pub id: i64,
    pub query: String,
    pub normalized_query: String,
    pub display_name: String,
    pub country_code: Option<String>,
    pub latitude: Option<String>,
    pub longitude: Option<String>,
    pub is_valid: bool,
    pub created_at: NaiveDateTime,
    pub expires_at: NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CustomerFilterQuery {
    pub q: Option<String>,
    pub status: Option<String>,
    pub verified: Option<String>,
    pub payment: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CustomerSummaryRow {
    pub customer_id: i64,
    pub user_id: i64,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub city: String,
    pub email_verified: bool,
    pub is_active: bool,
    pub appointment_count: i64,
    pub last_appointment_at: Option<NaiveDateTime>,
    pub next_appointment_at: Option<NaiveDateTime>,
    pub total_value_cents: i64,
    pub total_paid_cents: i64,
    pub total_open_cents: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct AppointmentOverviewRow {
    pub id: i64,
    pub desired_at: NaiveDateTime,
    pub status: String,
    pub message: Option<String>,
    pub total_amount_cents: i64,
    pub amount_paid_cents: i64,
    pub amount_open_cents: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct DashboardMetricsRow {
    pub active_customers: i64,
    pub future_appointments: i64,
    pub open_amount_cents: i64,
    pub unpaid_customers: i64,
}
