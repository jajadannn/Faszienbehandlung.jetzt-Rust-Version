# faszienbehandlung.jetzt

Produktionsnahe Praxis-Webbasis in Rust mit Axum, Askama, SQLx und SQLite fuer die Domain `https://www.faszienbehandlung.jetzt`.

## Architektur

- Web-Stack: Rust, Axum, serverseitiges Rendering mit Askama, SQLx fuer den Datenzugriff.
- Schichten: Routing, Handler, Auth, Datenbank, Services, Middleware, Templates, Assets.
- Sicherheit: Argon2-Passwort-Hashing, serverseitige Validierung, CSRF-Schutz per Double-Submit-Cookie, serverseitige Sessions, Rollenmodell fuer `admin` und `customer`.
- Datenschutz: datensparsame Registrierung, Wohnortvalidierung ueber Geocoding mit Cache, vorbereitete Rechtstexte als Platzhalter mit deutlicher Pruefpflicht.
- Buchung: Terminanfrage mit Kundenkonto, E-Mail-Verifizierung, Zahlungsbasis je Termin, Kundenbereich und Admin-Verwaltung.

## Projektstruktur

```text
.
|-- Cargo.toml
|-- .env.example
|-- migrations/
|   `-- 0001_initial.sql
|-- src/
|   |-- main.rs
|   |-- lib.rs
|   |-- config.rs
|   |-- db.rs
|   |-- error.rs
|   |-- auth.rs
|   |-- forms.rs
|   |-- models.rs
|   |-- content.rs
|   |-- views.rs
|   |-- seo.rs
|   |-- seed.rs
|   |-- state.rs
|   |-- utils.rs
|   |-- handlers/
|   |   |-- mod.rs
|   |   |-- pages.rs
|   |   |-- auth.rs
|   |   |-- booking.rs
|   |   |-- account.rs
|   |   `-- admin.rs
|   |-- middleware/
|   |   `-- security_headers.rs
|   `-- services/
|       |-- email.rs
|       `-- location.rs
|-- templates/
|   |-- layouts/base.html
|   `-- pages/*.html
|-- static/
|   |-- css/site.css
|   |-- js/site.js
|   |-- favicon.svg
|   `-- og/og-image.svg
`-- data/
```

## Datenmodell

Mindestens enthalten und umgesetzt:

- `users`
- `customers`
- `email_verifications`
- `sessions`
- `appointments`
- `payments`
- `payment_events`
- `admin_notes`
- `locations_validation_cache`

Die Tabellen sind in [`migrations/0001_initial.sql`](/workspace/migrations/0001_initial.sql) definiert und bilden eine realistische Grundlage fuer spaetere Erweiterungen.

## Lokaler Start

1. `.env.example` nach `.env` kopieren und bei Bedarf Werte anpassen.
2. Rust-Toolchain installieren.
3. Anwendung starten:

```bash
cargo run
```

4. Optional Demo-Daten einspielen:

```bash
cargo run -- seed-demo
```

Danach ist die App standardmaessig unter `http://127.0.0.1:3000` erreichbar.

Wichtig: Aenderungen an `.env` werden erst nach einem Neustart von `cargo run` sichtbar, weil die Konfiguration beim Start geladen wird.

## Wichtige Praxiswerte in `.env`

Diese Werte steuern die sichtbaren Praxisdaten und zentrale Inhalte der Startseite:

- `PRACTICE_NAME`
- `PRACTITIONER_NAME`
- `PRACTICE_EMAIL`
- `PRACTICE_PHONE`
- `PRACTICE_ADDRESS_LINE_1`
- `PRACTICE_ADDRESS_LINE_2`
- `PRACTICE_REGION_LABEL`
- `PRACTICE_HOUSE_CALL_AREA`
- `OPENING_HOURS_WEEKDAYS`
- `OPENING_HOURS_SATURDAY`
- `APPOINTMENT_DURATION_MINUTES`
- `BOOKING_BASE_PRICE_CENTS`
- `BOOKING_PACKAGE_SESSION_PRICE_CENTS`
- `BOOKING_PACKAGE_SESSION_COUNT`
- `BOOKING_PACKAGE_VALIDITY_MONTHS`
- `HOUSE_CALL_FEE_CENTS`

## Demo-Zugaenge nach `seed-demo`

- Admin: `admin@faszienbehandlung.jetzt`
- Kunde: `anna.beispiel@faszienbehandlung.jetzt`
- Kunde: `markus.demo@faszienbehandlung.jetzt`
- Kunden-Passwort: `KundeSicher!2026`
- Admin-Passwort: `PraxisAdmin!2026`

## E-Mail-Verifizierung

- Registrierung und Gast-Buchung erzeugen einen Datensatz in `email_verifications`.
- Tokens werden gehasht gespeichert.
- Der Link verweist auf `/verify-email?token=...`.
- Nach erfolgreicher Bestaetigung werden wartende Buchungen von `wartet_auf_email` auf `angefragt` gesetzt.

## Wohnortvalidierung

- Die Validierung nutzt OpenStreetMap Nominatim via `reqwest`.
- Nur der benoetigte Wohnort-String wird zur Pruefung verwendet.
- Treffer werden in `locations_validation_cache` zwischengespeichert, um Datenminimierung und Performance zu unterstuetzen.
- Vor Livegang sollten Nutzungsbedingungen, Rate Limits und ein stabiler produktiver Geocoding-Anbieter geprueft werden.

## Deployment-Hinweise

- Fuer den Produktivbetrieb ist PostgreSQL weiterhin eine sinnvolle Zielplattform. Die Anwendung ist aktuell lokal-first mit SQLite aufgebaut.
- TLS/Reverse-Proxy sauber konfigurieren, z. B. mit Caddy oder Nginx.
- `SESSION_COOKIE_SECURE=true` in Produktion setzen.
- SMTP fuer echte Transaktionsmails aktivieren.
- Backups, Monitoring, strukturierte Logs und Secret-Management vorsehen.
- Vor Livegang Rate Limiting, Fehlerseiten, Mail-Queues und Passwort-Reset-Flows weiter ausbauen.

## Was vor echtem Livegang geprueft werden muss

- Impressum und Datenschutzerklaerung fachlich und rechtlich vervollstaendigen.
- Tatsaechliche Berufsbezeichnung, Aufsichtsangaben und Anbieterkennzeichnung ergaenzen.
- Medizinische Aussagen, FAQ und Leistungsbeschreibungen inhaltlich freigeben.
- Echte Zahlungsprozesse, Storno-Regeln, Ausfallhonorare und Rechnungslogik definieren.
- Echte Verfuegbarkeitslogik fuer Termin-Slots ergaenzen.
- SMTP, Geocoding-Anbieter, Hosting, AV-Vertraege und Speicherfristen pruefen.
- Zugriffsschutz fuer Admin, Logging-Policies und Backup-Wiederherstellung testen.
