param(
    [string]$DeployRoot = "",
    [string]$EnvFile = "",
    [string]$ServiceName = "",
    [string]$ExecutableName = "faszienbehandlung_jetzt.exe"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

if ([string]::IsNullOrWhiteSpace($DeployRoot)) {
    if (-not [string]::IsNullOrWhiteSpace($env:DEPLOY_ROOT)) {
        $DeployRoot = $env:DEPLOY_ROOT
    } else {
        $DeployRoot = Join-Path $repoRoot "deploy"
    }
}

if ([string]::IsNullOrWhiteSpace($EnvFile) -and -not [string]::IsNullOrWhiteSpace($env:DEPLOY_ENV_FILE)) {
    $EnvFile = $env:DEPLOY_ENV_FILE
}

if ([string]::IsNullOrWhiteSpace($ServiceName) -and -not [string]::IsNullOrWhiteSpace($env:WINDOWS_SERVICE_NAME)) {
    $ServiceName = $env:WINDOWS_SERVICE_NAME
}

if (-not [string]::IsNullOrWhiteSpace($env:APP_EXECUTABLE_NAME)) {
    $ExecutableName = $env:APP_EXECUTABLE_NAME
}

$releaseBinary = Join-Path $repoRoot ("target\release\" + $ExecutableName)

if (-not (Test-Path $releaseBinary)) {
    throw "Release-Binary nicht gefunden: $releaseBinary"
}

$deployRootResolved = [System.IO.Path]::GetFullPath($DeployRoot)
$currentRoot = Join-Path $deployRootResolved "current"
$logsRoot = Join-Path $currentRoot "logs"
$dataRoot = Join-Path $currentRoot "data"

New-Item -ItemType Directory -Force -Path $currentRoot | Out-Null
New-Item -ItemType Directory -Force -Path $logsRoot | Out-Null
New-Item -ItemType Directory -Force -Path $dataRoot | Out-Null

$shouldRestartService = -not [string]::IsNullOrWhiteSpace($ServiceName)

if ($shouldRestartService) {
    $service = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue
    if ($null -ne $service -and $service.Status -ne "Stopped") {
        Stop-Service -Name $ServiceName -Force
        $service.WaitForStatus("Stopped", (New-TimeSpan -Seconds 30))
    }
} else {
    Get-Process -Name ([System.IO.Path]::GetFileNameWithoutExtension($ExecutableName)) -ErrorAction SilentlyContinue |
        Stop-Process -Force -ErrorAction SilentlyContinue
}

Copy-Item $releaseBinary (Join-Path $currentRoot $ExecutableName) -Force

foreach ($folder in @("templates", "static", "migrations")) {
    $source = Join-Path $repoRoot $folder
    $target = Join-Path $currentRoot $folder

    if (Test-Path $target) {
        Remove-Item -Path $target -Recurse -Force
    }

    Copy-Item $source $target -Recurse -Force
}

if (-not [string]::IsNullOrWhiteSpace($EnvFile) -and (Test-Path $EnvFile)) {
    Copy-Item $EnvFile (Join-Path $currentRoot ".env") -Force
} else {
    Write-Host "Keine externe .env gefunden. Lege fuer produktives Deployment DEPLOY_ENV_FILE fest."
}

if ($shouldRestartService) {
    Start-Service -Name $ServiceName
    Write-Host "Deployment abgeschlossen. Windows-Service '$ServiceName' wurde neu gestartet."
    exit 0
}

$stdoutLog = Join-Path $logsRoot "stdout.log"
$stderrLog = Join-Path $logsRoot "stderr.log"

Start-Process `
    -FilePath (Join-Path $currentRoot $ExecutableName) `
    -WorkingDirectory $currentRoot `
    -WindowStyle Hidden `
    -RedirectStandardOutput $stdoutLog `
    -RedirectStandardError $stderrLog

Write-Host "Deployment abgeschlossen. Prozess wurde direkt aus '$currentRoot' neu gestartet."
