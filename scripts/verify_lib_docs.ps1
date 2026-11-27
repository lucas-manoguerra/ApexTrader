
$ErrorActionPreference = "Stop"
$exitCode = 0
$repoRoot = Split-Path -Parent $PSScriptRoot
$cargoTomlPath = Join-Path $repoRoot "Cargo.toml"
$tasksDir = Join-Path $repoRoot "odd/tasks"

Write-Host "=== Verificación de documentación de librerías — ApexTrader ===" -ForegroundColor Cyan

if (-not (Test-Path -LiteralPath $cargoTomlPath -PathType Leaf)) {
    Write-Error "No se encontró Cargo.toml en: $cargoTomlPath"
    exit 1
}

$cargoToml = Get-Content -LiteralPath $cargoTomlPath -Raw
$declaredDeps = @{}
$insideDependencySection = $false

foreach ($line in ($cargoToml -split '\r?\n')) {
    if ($line -match '^\s*\[(dependencies|dev-dependencies|build-dependencies)(\.[^\]]+)?\]\s*$') {
        $insideDependencySection = $true
        continue
    }
    if ($line -match '^\s*\[') {
        $insideDependencySection = $false
        continue
    }
    if (-not $insideDependencySection -or $line -match '^\s*#' -or $line -match '^\s*$') {
        continue
    }

    if ($line -match '^\s*([a-zA-Z0-9_-]+)\s*=\s*(?:"([^"]+)"|\{[^}]*\bversion\s*=\s*"([^"]+)")') {
        $crate = $matches[1]
        $version = if ($matches[2]) { $matches[2] } else { $matches[3] }
        $declaredDeps[$crate] = $version
    }
}

Write-Host "Dependencias declaradas en Cargo.toml:" -ForegroundColor Yellow
foreach ($dependency in ($declaredDeps.Keys | Sort-Object)) {
    Write-Host "  $dependency = $($declaredDeps[$dependency])"
}

if (-not (Test-Path -LiteralPath $tasksDir -PathType Container)) {
    Write-Host "No existe odd/tasks/: no hay tareas ODD que verificar." -ForegroundColor Yellow
    exit 0
}

$taskFiles = Get-ChildItem -LiteralPath $tasksDir -Filter "*.md" -File -Recurse
foreach ($taskFile in $taskFiles) {
    $content = Get-Content -LiteralPath $taskFile.FullName -Raw
    $mentionsCrates = $false

    foreach ($crate in $declaredDeps.Keys) {
        if ($content -match "(?i)\b$([regex]::Escape($crate))\b") {
            $mentionsCrates = $true
            break
        }
    }

    if ($mentionsCrates) {
        $hasDocRef = $content -match "(?i)(docs\.rs|documentación oficial|librerías consultadas|versión consultada|APIs usadas)"
        if (-not $hasDocRef) {
            Write-Host "Tarea '$($taskFile.BaseName)' menciona dependencias sin registrar documentación consultada." -ForegroundColor Red
            Write-Host "  Registrar crates, versión, documentación oficial y APIs usadas." -ForegroundColor Red
            $exitCode = 1
        } else {
            Write-Host "Tarea '$($taskFile.BaseName)': OK — registra documentación." -ForegroundColor Green
        }
    }
}

if ($exitCode -eq 0) {
    Write-Host ""
    Write-Host "=== Verificación de documentación completada ===" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "=== INCONSISTENCIAS ENCONTRADAS ===" -ForegroundColor Red
}
exit $exitCode
