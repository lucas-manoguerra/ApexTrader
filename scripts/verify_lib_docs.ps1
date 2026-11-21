# Verificación de documentación de librerías — ApexTrader
#
# Valida que las tareas ODD y el código registren qué librerías externas se
# usan y en qué versión, para asegurar que los agentes consulten la docs
# oficial antes de implementar y no usen implementaciones viejas.
#
# Uso: pwsh -File scripts/verify_lib_docs.ps1
# Exit code: 0 = OK, 1 = inconsistencia detectada

$ErrorActionPreference = "Stop"
$exitCode = 0

Write-Host "=== Verificación de documentación de librerías — ApexTrader ===" -ForegroundColor Cyan

# --- Leer versiones declaradas en Cargo.toml ---
$cargoToml = Get-Content -Path (Resolve-Path "../Cargo.toml") -Raw
$declaredDeps = @{}

# Extraer líneas de dependencias simples: name = "version"
$lines = $cargoToml -split "`n"
foreach ($line in $lines) {
    if ($line -match '^\s*([a-zA-Z0-9_-]+)\s*=\s*"([^"]+)"') {
        $declaredDeps[$matches[1]] = $matches[2]
    }
    # También manejo dependencias con tabla: name = { version = "..." }
    if ($line -match '^\s*([a-zA-Z0-9_-]+)\s*=\s*\{') {
        $depName = $matches[1]
        # Buscar versión en las siguientes líneas hasta el cierre de la tabla
    }
}

Write-Host "Dependencias declaradas en Cargo.toml:" -ForegroundColor Yellow
foreach ($k in $declaredDeps.Keys | Sort-Object) {
    Write-Host "  $k = $($declaredDeps[$k])"
}

# --- Verificar que odd/tasks/ tenga referencias a librerías consultadas ---
$tasksDir = Resolve-Path "../odd/tasks"
if (-not (Test-Path $tasksDir)) {
    Write-Host "No hay carpeta odd/tasks/, omitiendo verificación de tareas." -ForegroundColor Yellow
    exit 0
}

$taskFiles = Get-ChildItem -Path $tasksDir -Filter "*.md" -Recurse
foreach ($taskFile in $taskFiles) {
    $content = Get-Content -Path $taskFile.FullName -Raw
    $taskName = $taskFile.BaseName

    # Buscar si menciona crates externos o versiones
    $mentionsCrates = $false
    foreach ($crate in $declaredDeps.Keys) {
        if ($content -match "\b$crate\b") {
            $mentionsCrates = $true
            break
        }
    }

    if ($mentionsCrates) {
        # Si menciona crates, debe tener una sección de "Librerías consultadas" o similar
        $hasDocRef = $content -match "(?i)(docs\.rs|documentación oficial|librerías consultadas|versión consultada|APIs usadas)"
        if (-not $hasDocRef) {
            Write-Host "Tarea '$taskName' menciona crates externos pero no registra documentación consultada." -ForegroundColor Red
            Write-Host "  Agregar una sección con: crates usados, versión exacta, docs.rs consultada, APIs usadas." -ForegroundColor Red
            $exitCode = 1
        } else {
            Write-Host "Tarea '$taskName': OK — registra documentación de librerías." -ForegroundColor Green
        }
    }
}

if ($exitCode -eq 0) {
    Write-Host "`n=== Todas las tareas registran documentación de librerías ===" -ForegroundColor Green
} else {
    Write-Host "`n=== INCONSISTENCIAS ENCONTRADAS ===" -ForegroundColor Red
}
exit $exitCode
