# Kody - Script de Instalacion para Windows PowerShell
# Un solo comando: irm https://raw.githubusercontent.com/yokonad/kody/main/install.ps1?t=$(Get-Date -Format "yyyyMMddHHmmss") | iex

$host.UI.RawUI.WindowTitle = "Kody - Instalacion"

Write-Host ""
Write-Host "KODY - Scanner de Vulnerabilidades CLI" -ForegroundColor Cyan
Write-Host ""

function Test-Command($cmd) {
    try {
        Get-Command $cmd -ErrorAction Stop | Out-Null
        return $true
    } catch {
        return $false
    }
}

function Pause-Script {
    Write-Host ""
    Write-Host "Presiona ENTER para salir..." -ForegroundColor Gray
    $null = Read-Host
}

# =============================================================================
# PRE-INSTALACION
# =============================================================================
Write-Host "[PRE-INSTALACION] Preparando..." -ForegroundColor Magenta

$BinDir = "$env:LOCALAPPDATA\bin\kody"
$BinPath = "$BinDir\kody.exe"

Write-Host "  [INFO] Directorio destino: $BinDir" -ForegroundColor Cyan

Write-Host ""

# =============================================================================
# PASO 1: Descargar
# =============================================================================
Write-Host "[PASO 1] Descargando Kody..." -ForegroundColor Magenta

$downloadUrl = "https://github.com/yokonad/kody/releases/latest/download/kody-x86_64-pc-windows-msvc.zip"
$tempZip = "$env:TEMP\kody-install.zip"
$tempExtract = "$env:TEMP\kody-install"

Write-Host "  [INFO] URL: $downloadUrl" -ForegroundColor DarkCyan
Write-Host "  [INFO] Descargando... (esto toma unos segundos)" -ForegroundColor Cyan

$ProgressPreference = 'Continue'

try {
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip -UseBasicParsing -TimeoutSec 120
    Write-Host "  [OK] Descarga completada!" -ForegroundColor Green
} catch {
    $statusCode = $_.Exception.Response.StatusCode.value__
    if ($statusCode -eq 404) {
        Write-Host "  [ERROR] No se encontro una version pre-compilada de Kody (HTTP 404)." -ForegroundColor Red
        Write-Host "  [INFO] Asegurate de que exista al menos un release en GitHub." -ForegroundColor Cyan
        Write-Host "  [INFO] Releases: https://github.com/yokonad/kody/releases" -ForegroundColor Cyan
        Write-Host ""
        Write-Host "  [INFO] Como alternativa, compila desde codigo fuente:" -ForegroundColor Yellow
        Write-Host "    git clone https://github.com/yokonad/kody.git" -ForegroundColor White
        Write-Host "    cd kody/kody" -ForegroundColor White
        Write-Host "    cargo build --release" -ForegroundColor White
    } else {
        Write-Host "  [ERROR] Error al descargar Kody." -ForegroundColor Red
        Write-Host "  [INFO] Motivo: $_" -ForegroundColor DarkCyan
        Write-Host "  [INFO] Verifica tu conexion a internet y que GitHub este accesible." -ForegroundColor Cyan
        Write-Host ""
        Write-Host "  [INFO] Como alternativa, compila desde codigo fuente:" -ForegroundColor Yellow
        Write-Host "    git clone https://github.com/yokonad/kody.git" -ForegroundColor White
        Write-Host "    cd kody/kody" -ForegroundColor White
        Write-Host "    cargo build --release" -ForegroundColor White
    }
    Pause-Script
    return
}

Write-Host ""

# =============================================================================
# PASO 2: Instalar
# =============================================================================
Write-Host "[PASO 2] Instalando Kody..." -ForegroundColor Magenta

# Limpiar instalacion anterior
if (Test-Path $BinDir) {
    Write-Host "  [INFO] Limpiando instalacion anterior..." -ForegroundColor Cyan
    try {
        Remove-Item -Recurse -Force $BinDir -ErrorAction Stop
        Write-Host "  [OK] Limpiado correctamente" -ForegroundColor Green
    } catch {
        Write-Host "  [WARN] No se pudo limpiar $BinDir : $_" -ForegroundColor Yellow
        Write-Host "  [INFO] Continuando... (puede que necesites cerrar kody.exe si esta abierto)" -ForegroundColor Cyan
    }
}

# Extraer archivo zip
Write-Host "  [INFO] Extrayendo..." -ForegroundColor Cyan

try {
    if (Test-Path $tempExtract) {
        Remove-Item -Recurse -Force $tempExtract -ErrorAction SilentlyContinue
    }

    Expand-Archive -Path $tempZip -DestinationPath $tempExtract -Force

    # Create install directory
    if (-not (Test-Path $BinDir)) {
        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    }

    # Copy binary — search for kody.exe in extraction dir (handle nested folders)
    $exe = Get-ChildItem -Path $tempExtract -Filter "kody.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1

    if ($exe) {
        Copy-Item $exe.FullName $BinPath -Force
        Write-Host "  [OK] Binario instalado en: $BinPath" -ForegroundColor Green
        $exeSize = (Get-Item $BinPath).Length / 1MB
        Write-Host "  [INFO] Tamano: $([math]::Round($exeSize, 2)) MB" -ForegroundColor Cyan
    } else {
        Write-Host "  [ERROR] No se encontro kody.exe en el archivo descargado." -ForegroundColor Red
        Write-Host "  [INFO] El archivo puede estar corrupto. Intenta de nuevo." -ForegroundColor Cyan
        Pause-Script
        return
    }
} catch {
    Write-Host "  [ERROR] Error al extraer: $_" -ForegroundColor Red
    Write-Host "  [INFO] El archivo descargado puede estar corrupto. Intenta de nuevo." -ForegroundColor Cyan
    Pause-Script
    return
} finally {
    # Cleanup temp files
    Remove-Item $tempZip -Force -ErrorAction SilentlyContinue
    Remove-Item $tempExtract -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "  [INFO] Archivos temporales eliminados" -ForegroundColor DarkCyan
}

Write-Host ""

# =============================================================================
# PASO 3: PATH
# =============================================================================
Write-Host "[PASO 3] Configurando PATH..." -ForegroundColor Magenta

$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
    Write-Host "  [OK] PATH actualizado (agregado: $BinDir)" -ForegroundColor Green
    Write-Host "  [INFO] Abre una NUEVA terminal para usar kody" -ForegroundColor Cyan
} else {
    Write-Host "  [OK] PATH ya configurado" -ForegroundColor Green
}

Write-Host ""
Write-Host "INSTALACION COMPLETADA!" -ForegroundColor Green
Write-Host ""
Write-Host "Abre una NUEVA terminal PowerShell y ejecuta:" -ForegroundColor Cyan
Write-Host "  kody --help" -ForegroundColor White
Write-Host ""

Pause-Script
