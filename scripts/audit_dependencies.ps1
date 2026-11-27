
$ErrorActionPreference = "Stop"

Write-Host "=== Auditoría de dependencias — ApexTrader ===" -ForegroundColor Cyan

$requiredTools = @("cargo-audit", "cargo-deny")
$missingTools = @(
    $requiredTools | Where-Object {
        -not (Get-Command $_ -ErrorAction SilentlyContinue)
    }
)

if ($missingTools.Count -gt 0) {
    Write-Host "Faltan herramientas obligatorias: $($missingTools -join ', ')" -ForegroundColor Red
    Write-Host "Instalar con: cargo install cargo-audit; cargo install cargo-deny" -ForegroundColor Yellow
    exit 1
}

Write-Host ""
Write-Host "[1/2] Ejecutando cargo audit (vulnerabilidades conocidas)..." -ForegroundColor Yellow
& cargo audit
if ($LASTEXITCODE -ne 0) {
    Write-Host "cargo audit detectó vulnerabilidades o no pudo completar la auditoría." -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "cargo audit: sin vulnerabilidades conocidas." -ForegroundColor Green

Write-Host ""
Write-Host "[2/2] Ejecutando cargo deny check (licencias, advisories y dependencias yanked)..." -ForegroundColor Yellow
& cargo deny --config cargo-deny.toml --locked check
if ($LASTEXITCODE -ne 0) {
    Write-Host "cargo deny detectó problemas o no pudo completar la auditoría." -ForegroundColor Red
    exit $LASTEXITCODE
}
Write-Host "cargo deny: verificaciones completadas sin problemas." -ForegroundColor Green

Write-Host ""
Write-Host "=== Auditoría de dependencias completada ===" -ForegroundColor Cyan
exit 0
