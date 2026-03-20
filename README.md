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

## Lokale Auto-Aktualisierung

Fuer die lokale Entwicklung ist Auto-Reload vorbereitet:

1. `cargo-watch` einmalig installieren:

```bash
cargo install cargo-watch
```

2. Unter Windows mit dem beigelegten Watch-Skript starten:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\dev-watch.ps1
```

Unter Linux:

```bash
chmod +x ./scripts/dev-watch.sh
./scripts/dev-watch.sh
```

Dann werden `src`, `templates`, `static`, `migrations` und `.env` ueberwacht. Sobald sich beim Rebuild die Server-Instanz aendert, aktualisiert der Browser die geoeffnete Seite automatisch.

Wichtige Variablen:

- `AUTO_RELOAD_ENABLED=true|false`
- `AUTO_RELOAD_INTERVAL_MS=1200`

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

## Automatisches Deployment aus GitHub

Im Projekt ist eine GitHub-Actions-Basis fuer automatisches Deployment auf Windows- und Linux-Self-Hosted-Runnern enthalten:

- Windows-Workflow: `.github/workflows/deploy-self-hosted.yml`
- Windows-Deploy-Skript: `scripts/deploy-release.ps1`
- Linux-Workflow: `.github/workflows/deploy-self-hosted-linux.yml`
- Linux-Deploy-Skript: `scripts/deploy-release.sh`
- Beispiel fuer `systemd`: `deploy/systemd/faszienbehandlung_jetzt.service.example`

Typischer Ablauf:

1. Auf dem Zielserver einen GitHub Self-Hosted Runner fuer dieses Repository registrieren.
2. Im Repository unter `Settings -> Secrets and variables -> Actions -> Variables` setzen:
   - `DEPLOY_ROOT`
   - `DEPLOY_ENV_FILE`
   - optional `WINDOWS_SERVICE_NAME` fuer Windows
   - optional `SYSTEMD_SERVICE_NAME` fuer Linux
3. Bei jedem Push auf `main` baut GitHub Actions die Release-Binary und fuehrt das passende Deploy-Skript auf dem Runner aus.

Das Deploy-Skript:

- kopiert Binary, Templates, Static Assets und Migrationen nach `DEPLOY_ROOT/current`
- uebernimmt auf Wunsch eine externe `.env`
- kann unter Windows einen Windows-Service und unter Linux einen `systemd`-Service neu starten
- faellt ohne Service-Namen auf einen direkten Prozess-Neustart zurueck

Fuer produktive Stabilitaet ist auf beiden Plattformen ein echter Service weiterhin die bessere Wahl als ein loser Hintergrundprozess.

### Linux-Hinweise

- Der Self-Hosted Runner braucht Schreibrechte auf `DEPLOY_ROOT`.
- Wenn `SYSTEMD_SERVICE_NAME` genutzt wird, braucht der Runner zusaetzlich die Berechtigung, diesen Dienst neu zu starten.
- Das Beispiel unter `deploy/systemd/faszienbehandlung_jetzt.service.example` ist eine gute Basis fuer einen produktiven Linux-Dienst.

## Was vor echtem Livegang geprueft werden muss

- Impressum und Datenschutzerklaerung fachlich und rechtlich vervollstaendigen.
- Tatsaechliche Berufsbezeichnung, Aufsichtsangaben und Anbieterkennzeichnung ergaenzen.
- Medizinische Aussagen, FAQ und Leistungsbeschreibungen inhaltlich freigeben.
- Echte Zahlungsprozesse, Storno-Regeln, Ausfallhonorare und Rechnungslogik definieren.
- Echte Verfuegbarkeitslogik fuer Termin-Slots ergaenzen.
- SMTP, Geocoding-Anbieter, Hosting, AV-Vertraege und Speicherfristen pruefen.
- Zugriffsschutz fuer Admin, Logging-Policies und Backup-Wiederherstellung testen.
