use chrono::NaiveDateTime;

use crate::{models::AuthenticatedUser, seo::SeoMeta};

#[derive(Clone)]
pub struct PracticeView {
    pub name: String,
    pub practitioner_name: String,
    pub email: String,
    pub phone: String,
    pub address_line_1: String,
    pub address_line_2: String,
    pub region_label: String,
    pub house_call_area: String,
    pub opening_hours_weekdays: String,
    pub opening_hours_saturday: String,
    pub opening_hours_summary: String,
    pub appointment_duration_short: String,
    pub appointment_duration_verbose: String,
    pub single_session_price_label: String,
    pub single_session_price_input: String,
    pub package_session_price_label: String,
    pub package_session_price_input: String,
    pub package_card_label: String,
    pub package_validity_label: String,
    pub package_validity_short: String,
    pub package_savings_label: String,
    pub house_call_fee_label: String,
    pub house_call_fee_input: String,
    pub maps_query: String,
}

#[derive(Clone)]
pub struct NavUserView {
    pub full_name: String,
    pub role: String,
    pub email_verified: bool,
}

impl From<&AuthenticatedUser> for NavUserView {
    fn from(value: &AuthenticatedUser) -> Self {
        Self {
            full_name: value.full_name.clone(),
            role: value.role.clone(),
            email_verified: value.email_verified,
        }
    }
}

#[derive(Clone)]
pub struct PageShell {
    pub meta: SeoMeta,
    pub current_user: Option<NavUserView>,
    pub csrf_token: String,
    pub year: i32,
    pub auto_reload_enabled: bool,
    pub auto_reload_interval_ms: u64,
    pub auto_reload_endpoint: String,
    pub server_instance_id: String,
    pub practice: PracticeView,
}

#[derive(Clone)]
pub struct FlashMessage {
    pub kind: String,
    pub title: String,
    pub text: String,
}

#[derive(Clone)]
pub struct ServiceCardView {
    pub title: String,
    pub subtitle: String,
    pub price: String,
    pub bullets: Vec<String>,
}

#[derive(Clone)]
pub struct FaqItemView {
    pub question: String,
    pub answer: String,
}

#[derive(Clone)]
pub struct AppointmentView {
    pub id: i64,
    pub desired_at: NaiveDateTime,
    pub desired_at_label: String,
    pub status: String,
    pub status_label: String,
    pub message: String,
    pub total_amount_label: String,
    pub paid_amount_label: String,
    pub open_amount_label: String,
}

#[derive(Clone)]
pub struct PaymentView {
    pub id: i64,
    pub amount_total_label: String,
    pub amount_paid_label: String,
    pub amount_open_label: String,
    pub status_label: String,
    pub payment_date_label: String,
    pub note: String,
}

#[derive(Clone)]
pub struct PaymentEventView {
    pub amount_label: String,
    pub created_at_label: String,
    pub note: String,
}

#[derive(Clone)]
pub struct NoteView {
    pub created_at_label: String,
    pub author_name: String,
    pub note: String,
}

#[derive(Clone)]
pub struct CustomerRowView {
    pub customer_id: i64,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub city: String,
    pub email_verified_label: String,
    pub status_label: String,
    pub appointment_count: String,
    pub last_appointment_label: String,
    pub next_appointment_label: String,
    pub total_paid_label: String,
    pub total_open_label: String,
}

#[derive(Clone)]
pub struct CustomerProfileView {
    pub customer_id: i64,
    pub full_name: String,
    pub email: String,
    pub phone_number: String,
    pub city: String,
    pub email_verified: bool,
    pub email_verified_label: String,
    pub status_label: String,
    pub appointment_count: String,
    pub last_appointment_label: String,
    pub next_appointment_label: String,
    pub total_paid_label: String,
    pub total_open_label: String,
    pub total_value_label: String,
}
