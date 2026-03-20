$ErrorActionPreference = "Stop"

Set-Location (Resolve-Path (Join-Path $PSScriptRoot ".."))

if (-not (Get-Command cargo-watch -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-watch ist noch nicht installiert."
    Write-Host "Bitte einmal ausfuehren: cargo install cargo-watch"
    exit 1
}

$env:AUTO_RELOAD_ENABLED = "true"

if (-not $env:AUTO_RELOAD_INTERVAL_MS) {
    $env:AUTO_RELOAD_INTERVAL_MS = "1200"
}

Write-Host "Starte Entwicklungsmodus mit automatischem Rebuild und Browser-Refresh..."

cargo watch `
    -w src `
    -w templates `
    -w static `
    -w migrations `
    -w .env `
    -x run
