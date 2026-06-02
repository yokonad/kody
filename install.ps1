# =============================================================================
# Kody - Script de Instalación para Windows
# =============================================================================
# Instalación con un solo comando (PowerShell):
#   irm https://raw.githubusercontent.com/yokonad/kody/main/install.ps1 | iex
# =============================================================================

$ErrorActionPreference = "Continue"

Write-Host ""
Write-Host "+============================================================+" -ForegroundColor Cyan
Write-Host "|                       KODY                               |" -ForegroundColor Red
Write-Host "|              Scanner de Vulnerabilidades CLI              |" -ForegroundColor Yellow
Write-Host "+============================================================+" -ForegroundColor Cyan
Write-Host ""

Write-Host "[INFO] Iniciando instalacion de Kody..." -ForegroundColor Cyan
Write-Host ""

# Verificar si el comando existe
function Test-Command($cmd) {
    $null = Get-Command $cmd -ErrorAction SilentlyContinue
    return $null -ne $null
}

# Función para pausar
function Pause {
    Write-Host "Presiona cualquier tecla para continuar..." -ForegroundColor Gray
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
}

# =============================================================================
# PASO 1: Instalar Rust
# =============================================================================
Write-Host "[PASO 1] Verificando Rust..." -ForegroundColor Magenta

if (Test-Command rustc) {
    $rustVersion = (rustc --version) -replace "rustc ", ""
    Write-Host "[OK] Rust ya esta instalado: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "[INFO] Rust no encontrado. Iniciando instalacion..." -ForegroundColor Yellow

    # Descargar rustup-init.exe
    $rustupUrl = "https://win.rustup.rs"
    $rustupPath = "$env:TEMP\rustup-init.exe"

    Write-Host "[INFO] Descargando rustup..." -ForegroundColor Yellow
    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -UseBasicParsing
    } catch {
        Write-Host "[ERROR] No se pudo descargar rustup: $_" -ForegroundColor Red
        Write-Host "[INFO] Descarga manualmente desde: https://rustup.rs" -ForegroundColor Cyan
        Pause
        exit 1
    }

    Write-Host "[INFO] Ejecutando instalador de Rust..." -ForegroundColor Yellow
    Write-Host "[INFO] Este proceso puede tomar varios minutos..." -ForegroundColor Gray

    try {
        Start-Process -FilePath $rustupPath -ArgumentList "-y", "--default-toolchain", "stable" -Wait -NoNewWindow
    } catch {
        Write-Host "[ERROR] Error al ejecutar rustup: $_" -ForegroundColor Red
        Pause
        exit 1
    }

    # Esperar un momento para que se complete la instalación
    Start-Sleep -Seconds 3

    # Refrescar entorno
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    # Verificar si se instaló
    Start-Sleep -Seconds 2
    if (Test-Command rustc) {
        $rustVersion = (rustc --version) -replace "rustc ", ""
        Write-Host "[OK] Rust instalado correctamente: $rustVersion" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] Rust no se pudo instalar correctamente." -ForegroundColor Red
        Write-Host "[INFO] Por favor, instala Rust manualmente desde: https://rustup.rs" -ForegroundColor Cyan
        Write-Host "[INFO] Luego reinicia PowerShell y ejecuta este script nuevamente." -ForegroundColor Cyan
        Pause
        exit 1
    }
}

Write-Host ""

# =============================================================================
# PASO 2: Clonar repositorio
# =============================================================================
Write-Host "[PASO 2] Descargando Kody..." -ForegroundColor Magenta

$KodyDir = "$HOME\kody"
$ProjectDir = "$KodyDir\kody"

if (Test-Path "$ProjectDir\.git") {
    Write-Host "[INFO] El repositorio ya existe. Actualizando..." -ForegroundColor Yellow
    Set-Location $ProjectDir
    git pull origin main 2>$null
} else {
    if (Test-Path $KodyDir) {
        Write-Host "[INFO] Eliminando directorio anterior..." -ForegroundColor Yellow
        Remove-Item -Recurse -Force $KodyDir
    }

    Write-Host "[INFO] Clonando repositorio..." -ForegroundColor Yellow
    git clone https://github.com/yokonad/kody.git $KodyDir
    Set-Location $ProjectDir
}

Write-Host "[OK] Repositorio listo" -ForegroundColor Green
Write-Host ""

# =============================================================================
# PASO 3: Compilar proyecto
# =============================================================================
Write-Host "[PASO 3] Compilando Kody..." -ForegroundColor Magenta
Write-Host "[INFO] Este proceso puede tomar de 5 a 15 minutos..." -ForegroundColor Gray
Write-Host "[INFO] En la primera compilacion se descargan todas las dependencias." -ForegroundColor Gray
Write-Host ""

# Refrescar PATH nuevamente
$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

# Verificar que cargo esté disponible
if (-not (Test-Command cargo)) {
    Write-Host "[ERROR] Cargo no esta disponible en el PATH." -ForegroundColor Red
    Write-Host "[INFO] Reinicia PowerShell y ejecuta: refreshenv" -ForegroundColor Cyan
    Write-Host "[INFO] O cierra y abre PowerShell nuevamente." -ForegroundColor Cyan
    Pause
    exit 1
}

try {
    cargo build --release 2>&1 | ForEach-Object { Write-Host "    $_" }
} catch {
    Write-Host "[ERROR] Error durante la compilacion: $_" -ForegroundColor Red
    Pause
    exit 1
}

if (Test-Path "target\release\kody.exe") {
    Write-Host ""
    Write-Host "[OK] Compilacion exitosa!" -ForegroundColor Green
} else {
    Write-Host "[ERROR] El archivo kody.exe no se encontro despues de la compilacion." -ForegroundColor Red
    Pause
    exit 1
}

Write-Host ""

# =============================================================================
# PASO 4: Instalar binary
# =============================================================================
Write-Host "[PASO 4] Instalando Kody..." -ForegroundColor Magenta

$BinDir = "$env:LOCALAPPDATA\bin\kody"
$BinPath = "$BinDir\kody.exe"

# Crear directorio si no existe
if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

# Copiar binario
Copy-Item "target\release\kody.exe" $BinPath -Force
Write-Host "[OK] Kody instalado en: $BinPath" -ForegroundColor Green

# Agregar al PATH del usuario
$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
    $env:Path = "$UserPath;$BinDir"
    Write-Host "[OK] $BinDir agregado al PATH" -ForegroundColor Green
}

Write-Host ""

# =============================================================================
# FINAL
# =============================================================================
Write-Host "+============================================================+" -ForegroundColor Green
Write-Host "|         INSTALACION COMPLETADA EXITOSAMENTE!              |" -ForegroundColor Green
Write-Host "+============================================================+" -ForegroundColor Green
Write-Host ""
Write-Host "Para usar Kody, cierra esta ventana y abre una Nueva terminal." -ForegroundColor Cyan
Write-Host "Luego ejecuta:" -ForegroundColor Cyan
Write-Host "  kody --help" -ForegroundColor White
Write-Host ""
Write-Host "O desde el directorio:" -ForegroundColor Cyan
Write-Host "  $ProjectDir\target\release\kody.exe --help" -ForegroundColor White
Write-Host ""
Write-Host "[NOTA] Si 'kody' no funciona, reinicia PowerShell." -ForegroundColor Yellow
Write-Host ""

Pause