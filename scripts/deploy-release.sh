#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

DEPLOY_ROOT="${DEPLOY_ROOT:-${REPO_ROOT}/deploy}"
DEPLOY_ENV_FILE="${DEPLOY_ENV_FILE:-}"
SYSTEMD_SERVICE_NAME="${SYSTEMD_SERVICE_NAME:-}"
APP_EXECUTABLE_NAME="${APP_EXECUTABLE_NAME:-faszienbehandlung_jetzt}"

RELEASE_BINARY="${REPO_ROOT}/target/release/${APP_EXECUTABLE_NAME}"

if [[ ! -f "${RELEASE_BINARY}" ]]; then
  echo "Release-Binary nicht gefunden: ${RELEASE_BINARY}" >&2
  exit 1
fi

CURRENT_ROOT="${DEPLOY_ROOT}/current"
LOGS_ROOT="${CURRENT_ROOT}/logs"
DATA_ROOT="${CURRENT_ROOT}/data"

mkdir -p "${CURRENT_ROOT}" "${LOGS_ROOT}" "${DATA_ROOT}"

SERVICE_MODE=false

if [[ -n "${SYSTEMD_SERVICE_NAME}" ]] && command -v systemctl >/dev/null 2>&1; then
  if systemctl show "${SYSTEMD_SERVICE_NAME}" --property=Id >/dev/null 2>&1; then
    SERVICE_MODE=true
    systemctl stop "${SYSTEMD_SERVICE_NAME}" || true
  fi
fi

if [[ "${SERVICE_MODE}" != "true" ]]; then
  pkill -f "${CURRENT_ROOT}/${APP_EXECUTABLE_NAME}" >/dev/null 2>&1 || true
fi

install -m 755 "${RELEASE_BINARY}" "${CURRENT_ROOT}/${APP_EXECUTABLE_NAME}"

for folder in templates static migrations; do
  rm -rf "${CURRENT_ROOT:?}/${folder}"
  cp -R "${REPO_ROOT}/${folder}" "${CURRENT_ROOT}/${folder}"
done

if [[ -n "${DEPLOY_ENV_FILE}" && -f "${DEPLOY_ENV_FILE}" ]]; then
  cp "${DEPLOY_ENV_FILE}" "${CURRENT_ROOT}/.env"
else
  echo "Keine externe .env gefunden. Lege fuer produktives Deployment DEPLOY_ENV_FILE fest."
fi

if [[ "${SERVICE_MODE}" == "true" ]]; then
  systemctl start "${SYSTEMD_SERVICE_NAME}"
  echo "Deployment abgeschlossen. systemd-Service '${SYSTEMD_SERVICE_NAME}' wurde neu gestartet."
  exit 0
fi

STDOUT_LOG="${LOGS_ROOT}/stdout.log"
STDERR_LOG="${LOGS_ROOT}/stderr.log"

nohup "${CURRENT_ROOT}/${APP_EXECUTABLE_NAME}" \
  >"${STDOUT_LOG}" \
  2>"${STDERR_LOG}" \
  </dev/null &

echo "Deployment abgeschlossen. Prozess wurde direkt aus '${CURRENT_ROOT}' neu gestartet."
