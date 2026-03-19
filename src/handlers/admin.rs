use askama::Template;
use axum::{
    Form,
    extract::{Path, Query, State},
    response::Response,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::NaiveDate;
use serde::Deserialize;
use sqlx::{FromRow, QueryBuilder, Sqlite};

use crate::{
    auth as auth_helpers,
    error::{AppError, AppResult},
    forms::{AdminAppointmentForm, AdminNoteForm, AdminPaymentForm, AppointmentStatusForm},
    models::{
        AppointmentOverviewRow, CustomerFilterQuery, CustomerSummaryRow, DashboardMetricsRow,
        Payment,
    },
    state::AppState,
    utils::{
        appointment_status_label, format_cents, format_datetime, infer_payment_status, now_utc,
        parse_datetime_local, parse_euro_to_cents, payment_status_label,
    },
    views::{
        AppointmentView, CustomerProfileView, CustomerRowView, FlashMessage, NoteView, PageShell,
        PaymentEventView, PaymentView,
    },
};

use super::{build_shell, redirect, render};

pub async fn dashboard(
    State(state): State<AppState>,
    jar: CookieJar,
    Query(filter): Query<CustomerFilterQuery>,
) -> AppResult<Response> {
    let current_user = auth_helpers::require_admin(auth_helpers::require_login(
        auth_helpers::load_current_user(&state, &jar).await?,
    )?)?;

    let metrics = sqlx::query_as::<_, DashboardMetricsRow>(
        r#"
        SELECT
            (SELECT COUNT(*) FROM customers WHERE is_active = 1) AS active_customers,
            (SELECT COUNT(*) FROM appointments WHERE desired_at > CURRENT_TIMESTAMP AND status != 'storniert') AS future_appointments,
            COALESCE((SELECT SUM(amount_open_cents) FROM payments), 0) AS open_amount_cents,
            (SELECT COUNT(DISTINCT customer_id) FROM payments WHERE amount_open_cents > 0) AS unpaid_customers
        "#,
    )
    .fetch_one(&state.pool)
    .await?;

    let customers = fetch_customer_rows(&state, &filter).await?;
    let customer_views = customers.into_iter().map(map_customer_row).collect();

    let (jar, shell, _) = build_shell(
        &state,
        jar,
        "/admin",
        "Admin | Kundenverwaltung, Termine und Zahlungen",
        "Professioneller Admin-Bereich fuer Kundenverwaltung, Terminstatus, Zahlungen, offene Betraege und interne Notizen.",
    )
    .await?;

    let template = AdminDashboardTemplate {
        shell,
        metrics: AdminMetricsView {
            active_customers: metrics.active_customers.to_string(),
            future_appointments: metrics.future_appointments.to_string(),
            open_amount: format_cents(metrics.open_amount_cents),
            unpaid_customers: metrics.unpaid_customers.to_string(),
        },
        customers: customer_views,
        filter: CustomerFilterView {
            q: filter.q.unwrap_or_default(),
            status: filter.status.unwrap_or_default(),
            verified: filter.verified.unwrap_or_default(),
            payment: filter.payment.unwrap_or_default(),
        },
        admin_name: current_user.full_name,
    };

    render(jar, &template)
}

pub async fn customer_detail(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(customer_id): Path<i64>,
) -> AppResult<Response> {
    let current_user = auth_helpers::require_admin(auth_helpers::require_login(
        auth_helpers::load_current_user(&state, &jar).await?,
    )?)?;

    render_customer_detail_page(&state, jar, customer_id, None, current_user.full_name).await
}

pub async fn add_note(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(customer_id): Path<i64>,
    Form(form): Form<AdminNoteForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;
    let admin = auth_helpers::require_admin(auth_helpers::require_login(
        auth_helpers::load_current_user(&state, &jar).await?,
    )?)?;

    let errors = form.validate();
    if !errors.is_empty() {
        return render_customer_detail_page(
            &state,
            jar,
            customer_id,
            Some(FlashMessage {
                kind: "error".to_string(),
                title: "Notiz konnte nicht gespeichert werden".to_string(),
                text: errors.join(" "),
            }),
            admin.full_name,
        )
        .await;
    }

    sqlx::query("INSERT INTO admin_notes (customer_id, admin_user_id, note, created_at) VALUES (?, ?, ?, ?)")
        .bind(customer_id)
        .bind(admin.user_id)
        .bind(form.note.trim())
        .bind(now_utc())
        .execute(&state.pool)
        .await?;

    Ok(redirect(jar, &format!("/admin/kunden/{}", customer_id)))
}

pub async fn add_appointment(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(customer_id): Path<i64>,
    Form(form): Form<AdminAppointmentForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;
    let admin = auth_helpers::require_admin(auth_helpers::require_login(
        auth_helpers::load_current_user(&state, &jar).await?,
    )?)?;

    let errors = form.validate();
    if !errors.is_empty() {
        return render_customer_detail_page(
            &state,
            jar,
            customer_id,
            Some(FlashMessage {
                kind: "error".to_string(),
                title: "Termin konnte nicht angelegt werden".to_string(),
                text: errors.join(" "),
            }),
            admin.full_name,
        )
        .await;
    }

    let desired_at = parse_datetime_local(&form.desired_at)?;
    let total_amount_cents = parse_euro_to_cents(&form.total_amount_eur)?;
    let now = now_utc();
    let mut tx = state.pool.begin().await?;

    let appointment_result = sqlx::query(
        r#"
        INSERT INTO appointments
            (customer_id, desired_at, status, message, total_amount_cents, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(customer_id)
    .bind(desired_at)
    .bind(form.status.trim())
    .bind(empty_to_none(&form.message))
    .bind(total_amount_cents)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;
    let appointment_id = appointment_result.last_insert_rowid();

    sqlx::query(
        r#"
        INSERT INTO payments
            (customer_id, appointment_id, amount_total_cents, amount_paid_cents, amount_open_cents, status, note, created_at, updated_at)
        VALUES (?, ?, ?, 0, ?, 'offen', ?, ?, ?)
        "#,
    )
    .bind(customer_id)
    .bind(appointment_id)
    .bind(total_amount_cents)
    .bind(total_amount_cents)
    .bind("Manuell im Admin-Bereich angelegt.")
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(redirect(jar, &format!("/admin/kunden/{}", customer_id)))
}

pub async fn add_payment(
    State(state): State<AppState>,
    jar: CookieJar,
    Path(customer_id): Path<i64>,
    Form(form): Form<AdminPaymentForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;
    let admin = auth_helpers::require_admin(auth_helpers::require_login(
        auth_helpers::load_current_user(&state, &jar).await?,
    )?)?;

    let errors = form.validate();
    if !errors.is_empty() {
        return render_customer_detail_page(
            &state,
            jar,
            customer_id,
            Some(FlashMessage {
                kind: "error".to_string(),
                title: "Zahlung konnte nicht verarbeitet werden".to_string(),
                text: errors.join(" "),
            }),
            admin.full_name,
        )
        .await;
    }

    let amount_total_cents = parse_euro_to_cents(&form.amount_total_eur)?;
    let delta_paid_cents = parse_euro_to_cents(&form.amount_paid_eur)?;
    if delta_paid_cents <= 0 {
        return render_customer_detail_page(
            &state,
            jar,
            customer_id,
            Some(FlashMessage {
                kind: "error".to_string(),
                title: "Zahlung konnte nicht verarbeitet werden".to_string(),
                text: "Bitte erfassen Sie einen positiven Zahlungseingang.".to_string(),
            }),
            admin.full_name,
        )
        .await;
    }

    let payment_date = if form.payment_date.trim().is_empty() {
        now_utc()
    } else {
        let parsed = NaiveDate::parse_from_str(&form.payment_date, "%Y-%m-%d").map_err(|_| {
            AppError::BadRequest("Bitte geben Sie ein gueltiges Zahlungsdatum an.".to_string())
        })?;
        parsed
            .and_hms_opt(12, 0, 0)
            .ok_or_else(|| AppError::BadRequest("Das Zahlungsdatum ist ungueltig.".to_string()))?
    };

    let now = now_utc();
    let mut tx = state.pool.begin().await?;

    let existing_payment = if let Some(appointment_id) = form.appointment_id {
        sqlx::query_as::<_, Payment>(
            "SELECT * FROM payments WHERE customer_id = ? AND appointment_id = ? ORDER BY id DESC LIMIT 1",
        )
        .bind(customer_id)
        .bind(appointment_id)
        .fetch_optional(&mut *tx)
        .await?
    } else {
        None
    };

    let payment_id = if let Some(payment) = existing_payment {
        let new_paid_total = payment.amount_paid_cents + delta_paid_cents;
        if new_paid_total > amount_total_cents {
            return render_customer_detail_page(
                &state,
                jar,
                customer_id,
                Some(FlashMessage {
                    kind: "error".to_string(),
                    title: "Zahlung konnte nicht verarbeitet werden".to_string(),
                    text: "Die neue Summe der Teilzahlungen uebersteigt den Gesamtbetrag."
                        .to_string(),
                }),
                admin.full_name,
            )
            .await;
        }

        let open_amount_cents = amount_total_cents - new_paid_total;
        let status = infer_payment_status(amount_total_cents, new_paid_total);

        sqlx::query(
            r#"
            UPDATE payments
            SET amount_total_cents = ?, amount_paid_cents = ?, amount_open_cents = ?, status = ?, payment_date = ?, note = ?, updated_at = ?
            WHERE id = ?
            "#,
        )
        .bind(amount_total_cents)
        .bind(new_paid_total)
        .bind(open_amount_cents)
        .bind(status)
        .bind(payment_date)
        .bind(empty_to_none(&form.note))
        .bind(now)
        .bind(payment.id)
        .execute(&mut *tx)
        .await?;

        payment.id
    } else {
        let open_amount_cents = amount_total_cents - delta_paid_cents;
        let status = infer_payment_status(amount_total_cents, delta_paid_cents);
        let payment_result = sqlx::query(
            r#"
            INSERT INTO payments
                (customer_id, appointment_id, amount_total_cents, amount_paid_cents, amount_open_cents, status, payment_date, note, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(customer_id)
        .bind(form.appointment_id)
        .bind(amount_total_cents)
        .bind(delta_paid_cents)
        .bind(open_amount_cents)
        .bind(status)
        .bind(payment_date)
        .bind(empty_to_none(&form.note))
        .bind(now)
        .bind(now)
        .execute(&mut *tx)
        .await?;
        payment_result.last_insert_rowid()
    };

    sqlx::query(
        "INSERT INTO payment_events (payment_id, recorded_by_user_id, amount_cents, note, created_at) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(payment_id)
    .bind(admin.user_id)
    .bind(delta_paid_cents)
    .bind(empty_to_none(&form.note))
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(redirect(jar, &format!("/admin/kunden/{}", customer_id)))
}

pub async fn update_appointment_status(
    State(state): State<AppState>,
    jar: CookieJar,
    Path((customer_id, appointment_id)): Path<(i64, i64)>,
    Form(form): Form<AppointmentStatusForm>,
) -> AppResult<Response> {
    auth_helpers::validate_csrf(&jar, &form.csrf_token)?;
    let admin = auth_helpers::require_admin(auth_helpers::require_login(
        auth_helpers::load_current_user(&state, &jar).await?,
    )?)?;

    if form.status.trim().is_empty() {
        return render_customer_detail_page(
            &state,
            jar,
            customer_id,
            Some(FlashMessage {
                kind: "error".to_string(),
                title: "Status konnte nicht aktualisiert werden".to_string(),
                text: "Bitte waehlen Sie einen gueltigen Status.".to_string(),
            }),
            admin.full_name,
        )
        .await;
    }

    sqlx::query(
        "UPDATE appointments SET status = ?, updated_at = ? WHERE id = ? AND customer_id = ?",
    )
    .bind(form.status.trim())
    .bind(now_utc())
    .bind(appointment_id)
    .bind(customer_id)
    .execute(&state.pool)
    .await?;

    Ok(redirect(jar, &format!("/admin/kunden/{}", customer_id)))
}

async fn render_customer_detail_page(
    state: &AppState,
    jar: CookieJar,
    customer_id: i64,
    flash: Option<FlashMessage>,
    admin_name: String,
) -> AppResult<Response> {
    let profile = sqlx::query_as::<_, CustomerSummaryRow>(
        r#"
        SELECT
            c.id AS customer_id,
            u.id AS user_id,
            u.full_name,
            u.email,
            u.phone_number,
            u.city,
            u.email_verified,
            c.is_active,
            (SELECT COUNT(*) FROM appointments a WHERE a.customer_id = c.id) AS appointment_count,
            (SELECT MAX(desired_at) FROM appointments a WHERE a.customer_id = c.id AND a.desired_at <= CURRENT_TIMESTAMP) AS last_appointment_at,
            (SELECT MIN(desired_at) FROM appointments a WHERE a.customer_id = c.id AND a.desired_at > CURRENT_TIMESTAMP AND a.status != 'storniert') AS next_appointment_at,
            COALESCE((SELECT SUM(amount_total_cents) FROM payments p WHERE p.customer_id = c.id), 0) AS total_value_cents,
            COALESCE((SELECT SUM(amount_paid_cents) FROM payments p WHERE p.customer_id = c.id), 0) AS total_paid_cents,
            COALESCE((SELECT SUM(amount_open_cents) FROM payments p WHERE p.customer_id = c.id), 0) AS total_open_cents
        FROM customers c
        JOIN users u ON u.id = c.user_id
        WHERE c.id = ?
        "#,
    )
    .bind(customer_id)
    .fetch_one(&state.pool)
    .await?;

    let appointments = sqlx::query_as::<_, AppointmentOverviewRow>(
        r#"
        SELECT
            a.id,
            a.desired_at,
            a.status,
            a.message,
            a.total_amount_cents,
            COALESCE(p.amount_paid_cents, 0) AS amount_paid_cents,
            COALESCE(p.amount_open_cents, a.total_amount_cents) AS amount_open_cents
        FROM appointments a
        LEFT JOIN payments p ON p.appointment_id = a.id
        WHERE a.customer_id = ?
        ORDER BY a.desired_at DESC
        "#,
    )
    .bind(customer_id)
    .fetch_all(&state.pool)
    .await?;

    let payments = sqlx::query_as::<_, Payment>(
        "SELECT * FROM payments WHERE customer_id = ? ORDER BY created_at DESC",
    )
    .bind(customer_id)
    .fetch_all(&state.pool)
    .await?;

    let payment_events = sqlx::query_as::<_, PaymentEventRow>(
        r#"
        SELECT pe.amount_cents, pe.note, pe.created_at
        FROM payment_events pe
        JOIN payments p ON p.id = pe.payment_id
        WHERE p.customer_id = ?
        ORDER BY pe.created_at DESC
        "#,
    )
    .bind(customer_id)
    .fetch_all(&state.pool)
    .await?;

    let notes = sqlx::query_as::<_, NoteRow>(
        r#"
        SELECT n.note, n.created_at, u.full_name AS author_name
        FROM admin_notes n
        JOIN users u ON u.id = n.admin_user_id
        WHERE n.customer_id = ?
        ORDER BY n.created_at DESC
        "#,
    )
    .bind(customer_id)
    .fetch_all(&state.pool)
    .await?;

    let appointment_views = appointments.into_iter().map(map_appointment_row).collect();
    let payment_views = payments.into_iter().map(map_payment_row).collect();
    let event_views = payment_events
        .into_iter()
        .map(|event| PaymentEventView {
            amount_label: format_cents(event.amount_cents),
            created_at_label: format_datetime(&event.created_at),
            note: event
                .note
                .unwrap_or_else(|| "Ohne zusaetzlichen Hinweis".to_string()),
        })
        .collect();
    let note_views = notes
        .into_iter()
        .map(|note| NoteView {
            created_at_label: format_datetime(&note.created_at),
            author_name: note.author_name,
            note: note.note,
        })
        .collect();

    let (jar, shell, _) = build_shell(
        state,
        jar,
        "/admin",
        "Admin | Kundenprofil",
        "Kundenprofil mit Terminverwaltung, Zahlungsstatus, Teilzahlungen und internen Notizen.",
    )
    .await?;

    let template = AdminCustomerTemplate {
        shell,
        admin_name,
        profile: CustomerProfileView {
            customer_id: profile.customer_id,
            full_name: profile.full_name,
            email: profile.email,
            phone_number: profile.phone_number,
            city: profile.city,
            email_verified_label: if profile.email_verified {
                "Ja".to_string()
            } else {
                "Nein".to_string()
            },
            status_label: if profile.is_active {
                "Aktiv".to_string()
            } else {
                "Inaktiv".to_string()
            },
            appointment_count: profile.appointment_count.to_string(),
            last_appointment_label: profile
                .last_appointment_at
                .map(|value| format_datetime(&value))
                .unwrap_or_else(|| "Noch kein abgeschlossener Termin".to_string()),
            next_appointment_label: profile
                .next_appointment_at
                .map(|value| format_datetime(&value))
                .unwrap_or_else(|| "Kein geplanter Termin".to_string()),
            total_paid_label: format_cents(profile.total_paid_cents),
            total_open_label: format_cents(profile.total_open_cents),
            total_value_label: format_cents(profile.total_value_cents),
        },
        appointments: appointment_views,
        payments: payment_views,
        payment_events: event_views,
        notes: note_views,
        flash,
    };

    render(jar, &template)
}

async fn fetch_customer_rows(
    state: &AppState,
    filter: &CustomerFilterQuery,
) -> AppResult<Vec<CustomerSummaryRow>> {
    let mut builder = QueryBuilder::<Sqlite>::new(
        r#"
        SELECT
            c.id AS customer_id,
            u.id AS user_id,
            u.full_name,
            u.email,
            u.phone_number,
            u.city,
            u.email_verified,
            c.is_active,
            (SELECT COUNT(*) FROM appointments a WHERE a.customer_id = c.id) AS appointment_count,
            (SELECT MAX(desired_at) FROM appointments a WHERE a.customer_id = c.id AND a.desired_at <= CURRENT_TIMESTAMP) AS last_appointment_at,
            (SELECT MIN(desired_at) FROM appointments a WHERE a.customer_id = c.id AND a.desired_at > CURRENT_TIMESTAMP AND a.status != 'storniert') AS next_appointment_at,
            COALESCE((SELECT SUM(amount_total_cents) FROM payments p WHERE p.customer_id = c.id), 0) AS total_value_cents,
            COALESCE((SELECT SUM(amount_paid_cents) FROM payments p WHERE p.customer_id = c.id), 0) AS total_paid_cents,
            COALESCE((SELECT SUM(amount_open_cents) FROM payments p WHERE p.customer_id = c.id), 0) AS total_open_cents
        FROM customers c
        JOIN users u ON u.id = c.user_id
        WHERE 1 = 1
        "#,
    );

    if let Some(q) = filter.q.as_ref().filter(|value| !value.trim().is_empty()) {
        let pattern = format!("%{}%", q.trim());
        builder.push(" AND (u.full_name LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR u.email LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR u.city LIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR u.phone_number LIKE ");
        builder.push_bind(pattern);
        builder.push(')');
    }

    if let Some(status) = filter.status.as_deref() {
        match status {
            "aktiv" => {
                builder.push(" AND c.is_active = 1");
            }
            "inaktiv" => {
                builder.push(" AND c.is_active = 0");
            }
            _ => {}
        }
    }

    if let Some(verified) = filter.verified.as_deref() {
        match verified {
            "ja" => {
                builder.push(" AND u.email_verified = 1");
            }
            "nein" => {
                builder.push(" AND u.email_verified = 0");
            }
            _ => {}
        }
    }

    if let Some(payment) = filter.payment.as_deref() {
        match payment {
            "offen" => {
                builder.push(
                    " AND EXISTS (SELECT 1 FROM payments p WHERE p.customer_id = c.id AND p.amount_open_cents > 0)",
                );
            }
            "bezahlt" => {
                builder.push(
                    " AND NOT EXISTS (SELECT 1 FROM payments p WHERE p.customer_id = c.id AND p.amount_open_cents > 0)",
                );
            }
            _ => {}
        }
    }

    builder.push(" ORDER BY u.full_name ASC");

    Ok(builder
        .build_query_as::<CustomerSummaryRow>()
        .fetch_all(&state.pool)
        .await?)
}

fn map_customer_row(row: CustomerSummaryRow) -> CustomerRowView {
    CustomerRowView {
        customer_id: row.customer_id,
        full_name: row.full_name,
        email: row.email,
        phone_number: row.phone_number,
        city: row.city,
        email_verified_label: if row.email_verified {
            "Ja".to_string()
        } else {
            "Nein".to_string()
        },
        status_label: if row.is_active {
            "Aktiv".to_string()
        } else {
            "Inaktiv".to_string()
        },
        appointment_count: row.appointment_count.to_string(),
        last_appointment_label: row
            .last_appointment_at
            .map(|value| format_datetime(&value))
            .unwrap_or_else(|| "Noch kein vergangener Termin".to_string()),
        next_appointment_label: row
            .next_appointment_at
            .map(|value| format_datetime(&value))
            .unwrap_or_else(|| "Kein geplanter Termin".to_string()),
        total_paid_label: format_cents(row.total_paid_cents),
        total_open_label: format_cents(row.total_open_cents),
    }
}

fn map_appointment_row(appointment: AppointmentOverviewRow) -> AppointmentView {
    AppointmentView {
        id: appointment.id,
        desired_at: appointment.desired_at,
        desired_at_label: format_datetime(&appointment.desired_at),
        status: appointment.status.clone(),
        status_label: appointment_status_label(&appointment.status).to_string(),
        message: appointment
            .message
            .unwrap_or_else(|| "Keine zusaetzliche Nachricht".to_string()),
        total_amount_label: format_cents(appointment.total_amount_cents),
        paid_amount_label: format_cents(appointment.amount_paid_cents),
        open_amount_label: format_cents(appointment.amount_open_cents),
    }
}

fn map_payment_row(payment: Payment) -> PaymentView {
    PaymentView {
        id: payment.id,
        amount_total_label: format_cents(payment.amount_total_cents),
        amount_paid_label: format_cents(payment.amount_paid_cents),
        amount_open_label: format_cents(payment.amount_open_cents),
        status_label: payment_status_label(&payment.status).to_string(),
        payment_date_label: payment
            .payment_date
            .map(|value| format_datetime(&value))
            .unwrap_or_else(|| "Noch kein Zahlungseingang".to_string()),
        note: payment
            .note
            .unwrap_or_else(|| "Kein Hinweis hinterlegt".to_string()),
    }
}

fn empty_to_none(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

#[derive(Debug, Clone, Deserialize)]
struct CustomerFilterView {
    q: String,
    status: String,
    verified: String,
    payment: String,
}

#[derive(Debug, Clone)]
struct AdminMetricsView {
    active_customers: String,
    future_appointments: String,
    open_amount: String,
    unpaid_customers: String,
}

#[derive(Debug, Clone, FromRow)]
struct PaymentEventRow {
    amount_cents: i64,
    note: Option<String>,
    created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, FromRow)]
struct NoteRow {
    note: String,
    created_at: chrono::NaiveDateTime,
    author_name: String,
}

#[derive(Template)]
#[template(path = "pages/admin_dashboard.html")]
struct AdminDashboardTemplate {
    shell: PageShell,
    metrics: AdminMetricsView,
    customers: Vec<CustomerRowView>,
    filter: CustomerFilterView,
    admin_name: String,
}

#[derive(Template)]
#[template(path = "pages/admin_customer.html")]
struct AdminCustomerTemplate {
    shell: PageShell,
    admin_name: String,
    profile: CustomerProfileView,
    appointments: Vec<AppointmentView>,
    payments: Vec<PaymentView>,
    payment_events: Vec<PaymentEventView>,
    notes: Vec<NoteView>,
    flash: Option<FlashMessage>,
}
