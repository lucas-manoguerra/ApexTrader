# Auditoría de dependencias — ApexTrader
#
# Ejecuta cargo-audit y cargo-deny si están instalados.
# Si una herramienta no está instalada, informa pero no falla el script
# (para no bloquear CI prematuramente antes de instalar las herramientas).
#
# Uso: pwsh -File scripts/audit_dependencies.ps1
# Exit code: 0 = OK, 1 = vulnerabilidades o licencias prohibidas encontradas

$ErrorActionPreference = "Continue"
$exitCode = 0

Write-Host "=== Auditoría de dependencias — ApexTrader ===" -ForegroundColor Cyan

# --- cargo-audit: vulnerabilidades conocidas ---
Write-Host "`n[1/2] Ejecutando cargo audit (vulnerabilidades)..." -ForegroundColor Yellow
$auditPath = Get-Command cargo-audit -ErrorAction SilentlyContinue
if ($auditPath) {
    $result = cargo audit --exit-code 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "VULNERABILIDADES ENCONTRADAS:" -ForegroundColor Red
        Write-Host $result -ForegroundColor Red
        $exitCode = 1
    } else {
        Write-Host "cargo audit: sin vulnerabilidades conocidas." -ForegroundColor Green
    }
} else {
    Write-Host "cargo-audit no está instalado. Instalar con: cargo install cargo-audit" -ForegroundColor Yellow
}

# --- cargo-deny: licencias, yanked, unmaintained ---
Write-Host "`n[2/2] Ejecutando cargo deny check (licencias, yanked, unmaintained)..." -ForegroundColor Yellow
$denyPath = Get-Command cargo-deny -ErrorAction SilentlyContinue
if ($denyPath) {
    $result = cargo deny check 2>&1
    if ($LASTEXITCODE -ne 0) {
        Write-Host "cargo deny detectó problemas:" -ForegroundColor Red
        Write-Host $result -ForegroundColor Red
        $exitCode = 1
    } else {
        Write-Host "cargo deny: sin problemas de licencias/vulnerabilidades." -ForegroundColor Green
    }
} else {
    Write-Host "cargo-deny no está instalado. Instalar con: cargo install cargo-deny" -ForegroundColor Yellow
}

Write-Host "`n=== Fin de auditoría ===" -ForegroundColor Cyan
exit $exitCode
