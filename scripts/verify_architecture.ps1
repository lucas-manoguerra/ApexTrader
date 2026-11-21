# Verificación de límites arquitectónicos — ApexTrader
#
# Comprueba que la capa de dominio (src/domain) no depende de crates de
# infraestructura (egui, wgpu, winit, raw_window_handle, etc.).
# Las dependencias de diseño deben apuntar hacia el dominio, nunca al revés.
#
# Uso: pwsh -File scripts/verify_architecture.ps1
# Exit code: 0 = OK, 1 = violación detectada

$ErrorActionPreference = "Stop"

$domainDir = Join-Path -Path $PSScriptRoot -ChildPath "../src/domain"
$domainDir = Resolve-Path -Path $domainDir

# Crates de infraestructura que NUNCA deben importarse desde domain/
$forbiddenCrates = @(
    "egui",
    "eframe",
    "wgpu",
    "wgpu_types",
    "winit",
    "raw_window_handle",
    "pollster",
    "bytemuck"
)

$violations = @()

foreach ($file in (Get-ChildItem -Path $domainDir -Filter "*.rs" -Recurse)) {
    $content = Get-Content -Path $file.FullName -Raw
    foreach ($crate in $forbiddenCrates) {
        # Detecta `use ...::egui` o `use egui::...` o `extern crate egui`
        if ($content -match "(use|extern\s+crate)\s+.*\b$crate\b" -or $content -match "\b$crate\b::") {
            $violations += "$($file.Path): importa de crate prohibido '$crate'"
        }
    }
}

if ($violations.Count -gt 0) {
    Write-Host "VIOLACIONES DE ARQUITECTURA DETECTADAS:" -ForegroundColor Red
    $violations | ForEach-Object { Write-Host "  - $_" -ForegroundColor Red }
    exit 1
} else {
    Write-Host "Arquitectura OK: src/domain/ no depende de crates de infraestructura." -ForegroundColor Green
    exit 0
}
