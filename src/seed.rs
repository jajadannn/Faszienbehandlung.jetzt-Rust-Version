use sqlx::Row;

use crate::{auth::hash_password, error::AppResult, state::AppState, utils::now_utc};

pub async fn seed_demo(state: &AppState) -> AppResult<()> {
    let existing: Option<(i64,)> = sqlx::query_as("SELECT id FROM users LIMIT 1")
        .fetch_optional(&state.pool)
        .await?;
    if existing.is_some() {
        tracing::info!("Seed übersprungen, da bereits Daten vorhanden sind.");
        return Ok(());
    }

    let now = now_utc();
    let admin_password = hash_password("PraxisAdmin!2026")?;
    let customer_password = hash_password("KundeSicher!2026")?;

    let mut tx = state.pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO users
            (full_name, email, email_verified, phone_number, city, password_hash, role, created_at, updated_at)
        VALUES
            (?, ?, 1, ?, ?, ?, 'admin', ?, ?)
        "#,
    )
    .bind("Praxis Admin")
    .bind("admin@faszienbehandlung.jetzt")
    .bind("+49 30 5555 120")
    .bind("Berlin")
    .bind(admin_password)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO users
            (full_name, email, email_verified, phone_number, city, password_hash, role, created_at, updated_at)
        VALUES
            (?, ?, 1, ?, ?, ?, 'customer', ?, ?),
            (?, ?, 1, ?, ?, ?, 'customer', ?, ?)
        "#,
    )
    .bind("Anna Beispiel")
    .bind("anna.beispiel@faszienbehandlung.jetzt")
    .bind("+49 170 100 2000")
    .bind("Potsdam")
    .bind(customer_password.clone())
    .bind(now)
    .bind(now)
    .bind("Markus Demo")
    .bind("markus.demo@faszienbehandlung.jetzt")
    .bind("+49 171 333 4400")
    .bind("Berlin")
    .bind(customer_password)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO customers (user_id, is_active, created_at, updated_at)
        SELECT id, 1, ?, ? FROM users WHERE role = 'customer'
        "#,
    )
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    let anna_customer_id: i64 = sqlx::query(
        "SELECT c.id FROM customers c JOIN users u ON u.id = c.user_id WHERE u.email = ?",
    )
    .bind("anna.beispiel@faszienbehandlung.jetzt")
    .fetch_one(&mut *tx)
    .await?
    .get(0);

    let markus_customer_id: i64 = sqlx::query(
        "SELECT c.id FROM customers c JOIN users u ON u.id = c.user_id WHERE u.email = ?",
    )
    .bind("markus.demo@faszienbehandlung.jetzt")
    .fetch_one(&mut *tx)
    .await?
    .get(0);

    sqlx::query(
        r#"
        INSERT INTO appointments
            (customer_id, desired_at, status, message, total_amount_cents, created_at, updated_at)
        VALUES
            (?, datetime('now', '+3 day'), 'bestaetigt', 'Bitte Fokus auf Nacken und rechte Schulter.', 8900, ?, ?),
            (?, datetime('now', '-18 day'), 'abgeschlossen', 'Folgetermin nach Lauftraining.', 7900, ?, ?)
        "#,
    )
    .bind(anna_customer_id)
    .bind(now)
    .bind(now)
    .bind(markus_customer_id)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    let anna_appointment_id: i64 =
        sqlx::query("SELECT id FROM appointments WHERE customer_id = ? ORDER BY id ASC LIMIT 1")
            .bind(anna_customer_id)
            .fetch_one(&mut *tx)
            .await?
            .get(0);

    let markus_appointment_id: i64 =
        sqlx::query("SELECT id FROM appointments WHERE customer_id = ? ORDER BY id DESC LIMIT 1")
            .bind(markus_customer_id)
            .fetch_one(&mut *tx)
            .await?
            .get(0);

    sqlx::query(
        r#"
        INSERT INTO payments
            (customer_id, appointment_id, amount_total_cents, amount_paid_cents, amount_open_cents, status, payment_date, note, created_at, updated_at)
        VALUES
            (?, ?, 8900, 4500, 4400, 'teilweise_bezahlt', datetime('now', '-1 day'), 'Anzahlung eingegangen.', ?, ?),
            (?, ?, 7900, 7900, 0, 'bezahlt', datetime('now', '-17 day'), 'Vor Ort bezahlt.', ?, ?)
        "#,
    )
    .bind(anna_customer_id)
    .bind(anna_appointment_id)
    .bind(now)
    .bind(now)
    .bind(markus_customer_id)
    .bind(markus_appointment_id)
    .bind(now)
    .bind(now)
    .execute(&mut *tx)
    .await?;

    let anna_payment_id: i64 =
        sqlx::query("SELECT id FROM payments WHERE customer_id = ? ORDER BY id ASC LIMIT 1")
            .bind(anna_customer_id)
            .fetch_one(&mut *tx)
            .await?
            .get(0);

    sqlx::query(
        "INSERT INTO payment_events (payment_id, amount_cents, note, created_at) VALUES (?, ?, ?, ?)",
    )
    .bind(anna_payment_id)
    .bind(4500_i64)
    .bind("Anzahlung bei Online-Bestätigung.")
    .bind(now)
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO admin_notes (customer_id, admin_user_id, note, created_at) VALUES (?, 1, ?, ?)",
    )
    .bind(anna_customer_id)
    .bind("Kundin wünscht bevorzugt Termine am späten Nachmittag.")
    .bind(now)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
