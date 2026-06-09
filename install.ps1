# Kody - Instalador para Windows (binario pre-compilado, ~10 segundos)
# Uso: irm https://raw.githubusercontent.com/yokonad/kody/main/install.ps1 | iex

$ErrorActionPreference = "Stop"

# ── Estetica GHOST ──────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  _  __ ___  ____  _   _ " -ForegroundColor Red
Write-Host " | |/ // _ \|  _ \| | | |" -ForegroundColor Red
Write-Host " | ' /| | | | | | | | | |" -ForegroundColor Red
Write-Host " | . \| |_| | |_| | |_| |" -ForegroundColor Red
Write-Host " |_|\_\\___/|____/ \___/ " -ForegroundColor Red
Write-Host "  private. dangerous. elite.   KODY installer" -ForegroundColor DarkGray
Write-Host ""

function Write-Step($label) {
    Write-Host ("[ {0} ]" -f $label) -ForegroundColor DarkGray -NoNewline
    Write-Host " ... [OK]" -ForegroundColor Green
}

Write-Step "establishing secure channel"
Write-Step "resolving latest release"

# ── Configuracion ───────────────────────────────────────────────────────────
$Url        = "https://github.com/yokonad/kody/releases/latest/download/kody-x86_64-pc-windows-msvc.zip"
$TmpDir     = "$env:TEMP\kody-install"
$ZipPath    = "$TmpDir\kody.zip"
$InstallDir = "$env:LOCALAPPDATA\bin\kody"
$BinPath    = "$InstallDir\kody.exe"

# Debug helper (DarkCyan, como en el script original)
function Write-DebugLine($msg) { Write-Host "  [debug] $msg" -ForegroundColor DarkCyan }

# ── PASO 1: Descargando binario pre-compilado ───────────────────────────────
Write-Host ""
Write-Host "[PASO 1] Descargando Kody (binario pre-compilado)..." -ForegroundColor Magenta
Write-DebugLine "origen: $Url"

if (Test-Path $TmpDir) { Remove-Item -Recurse -Force $TmpDir }
New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

# Barra de progreso de descarga
$ProgressPreference = "Continue"

try {
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing
}
catch {
    $status = $null
    try { $status = $_.Exception.Response.StatusCode.value__ } catch { }
    if ($status -eq 404) {
        Write-Host "  [ERROR] No se encontro una version pre-compilada (HTTP 404)." -ForegroundColor Red
        Write-Host "  Aun no hay un release publicado para tu plataforma." -ForegroundColor Yellow
        Write-Host "  Como alternativa, compila desde codigo fuente:" -ForegroundColor Cyan
        Write-Host "    git clone https://github.com/yokonad/kody.git" -ForegroundColor White
        Write-Host "    cd kody/kody; cargo build --release" -ForegroundColor White
    }
    else {
        Write-Host "  [ERROR] Fallo de red al descargar: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "  Revisa tu conexion e intenta de nuevo." -ForegroundColor Yellow
    }
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
    exit 1
}

Write-Host "  [OK] Descarga completada" -ForegroundColor Green

# ── PASO 2: Extraer e instalar ──────────────────────────────────────────────
Write-Host ""
Write-Host "[PASO 2] Instalando..." -ForegroundColor Magenta

Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$ExtractedExe = Get-ChildItem -Path $TmpDir -Filter "kody.exe" -Recurse | Select-Object -First 1
if (-not $ExtractedExe) {
    Write-Host "  [ERROR] No se encontro kody.exe dentro del archivo descargado." -ForegroundColor Red
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
    exit 1
}

Copy-Item -Path $ExtractedExe.FullName -Destination $BinPath -Force
Write-Host "  [OK] Kody instalado en: $BinPath" -ForegroundColor Green

# Limpieza de temporales
Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue

# ── PASO 3: Configurar PATH ─────────────────────────────────────────────────
Write-Host ""
Write-Host "[PASO 3] Configurando PATH..." -ForegroundColor Magenta

$UserPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$InstallDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$UserPath;$InstallDir", "User")
    $env:Path = "$env:Path;$InstallDir"
    Write-Host "  [OK] PATH actualizado" -ForegroundColor Green
}
else {
    Write-Host "  [OK] El PATH ya contenia la ruta" -ForegroundColor Green
}

# ── Listo ───────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "INSTALACION COMPLETADA!" -ForegroundColor Green
Write-Host ""
Write-Host "Abre una NUEVA terminal y ejecuta:" -ForegroundColor Cyan
Write-Host "  kody --help" -ForegroundColor White
Write-Host "  kody buscar ejemplo.com" -ForegroundColor White
Write-Host ""
