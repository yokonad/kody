# =============================================================================
# Kody - Script de Instalación para Windows
# =============================================================================
# Instalación con un solo comando (PowerShell):
#   irm https://raw.githubusercontent.com/yokonad/kody/main/install.ps1 | iex
# =============================================================================

$ErrorActionPreference = "Stop"

# Colores para la consola
function Write-Banner {
    Write-Host ""
    Write-Host "+============================================================+" -ForegroundColor Cyan
    Write-Host "|                                                          |" -ForegroundColor Cyan
    Write-Host "|                       KODY                               |" -ForegroundColor Red
    Write-Host "|                                                          |" -ForegroundColor Cyan
    Write-Host "|              Scanner de Vulnerabilidades CLI              |" -ForegroundColor Yellow
    Write-Host "|                    Desarrollado en Rust                   |" -ForegroundColor Yellow
    Write-Host "|                                                          |" -ForegroundColor Cyan
    Write-Host "+============================================================+" -ForegroundColor Cyan
    Write-Host ""
}

# Verificar si el comando existe
function Test-Command($cmd) {
    $null = Get-Command $cmd -ErrorAction SilentlyContinue
    return $null -ne $null
}

# Verificar e instalar Rust
function Install-Rust {
    if (Test-Command rustc) {
        $version = rustc --version | ForEach-Object { $_ -split ' ' | Select-Object -First 2 -Skip 1 }
        Write-Host "[OK] Rust ya esta instalado: $version" -ForegroundColor Green
        return
    }

    Write-Host "[INFO] Rust no encontrado. Instalando Rust..." -ForegroundColor Yellow

    # Instalar rustup via curl o descargando
    if (Test-Command curl) {
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    } elseif (Test-Command git) {
        # Descargar rustup-init.exe
        $rustupUrl = "https://win.rustup.rs"
        $rustupPath = "$env:TEMP\rustup-init.exe"

        Write-Host "[INFO] Descargando rustup..." -ForegroundColor Yellow
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath

        Write-Host "[INFO] Ejecutando rustup..." -ForegroundColor Yellow
        & $rustupPath -y

        Remove-Item $rustupPath -Force
    } else {
        Write-Host "[ERROR] Necesitas curl o git para instalar Rust." -ForegroundColor Red
        Write-Host "        Instala git desde: https://git-scm.com/download/win" -ForegroundColor Red
        exit 1
    }

    # Refrescar entorno
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    if (Test-Command rustc) {
        Write-Host "[OK] Rust instalado correctamente: $(rustc --version)" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] Error al instalar Rust." -ForegroundColor Red
        Write-Host "        Por favor, instala Rust manualmente desde https://rustup.rs" -ForegroundColor Red
        exit 1
    }
}

# Clonar o actualizar repositorio
function Get-Repo {
    $KodyDir = "$HOME\kody"

    if (Test-Path "$KodyDir\.git") {
        Write-Host "[INFO] Actualizando repositorio existente..." -ForegroundColor Yellow
        Set-Location $KodyDir
        git pull origin main 2>$null
    } else {
        Write-Host "[INFO] Clonando repositorio..." -ForegroundColor Yellow
        git clone https://github.com/yokonad/kody.git $KodyDir
        Set-Location $KodyDir
    }

    Write-Host "[OK] Repositorio listo en: $KodyDir" -ForegroundColor Green
}

# Compilar proyecto
function Build-Project {
    Write-Host "[INFO] Compilando proyecto (esto puede tomar varios minutos)..." -ForegroundColor Yellow

    Set-Location kody

    # Refrescar PATH
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    # Compilar
    cargo build --release 2>&1 | Out-Null

    if (Test-Path "target\release\kody.exe") {
        Write-Host "[OK] Compilacion exitosa!" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] Error en la compilacion." -ForegroundColor Red
        exit 1
    }
}

# Instalar binario
function Install-Binary {
    $BinDir = "$env:LOCALAPPDATA\bin\kody"
    $BinPath = "$BinDir\kody.exe"

    # Crear directorio si no existe
    if (-not (Test-Path $BinDir)) {
        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    }

    # Copiar binario
    Copy-Item "target\release\kody.exe" $BinPath -Force
    Write-Host "[OK] Kody instalado en: $BinPath" -ForegroundColor Green

    # Agregar al PATH del usuario si es necesario
    $UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
    if ($UserPath -notlike "*$BinDir*") {
        [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
        $env:Path = "$UserPath;$BinDir"
        Write-Host "[INFO] Se agrego $BinDir al PATH del usuario." -ForegroundColor Yellow
    }
}

# Función principal
function Main {
    Clear-Host
    Write-Banner

    Write-Host "[INFO] Iniciando instalacion de Kody..." -ForegroundColor Cyan
    Write-Host ""

    try {
        Install-Rust
        Get-Repo
        Build-Project
        Install-Binary

        Write-Host ""
        Write-Host "+============================================================+" -ForegroundColor Green
        Write-Host "|  [OK] Instalacion completada!                              |" -ForegroundColor Green
        Write-Host "+============================================================+" -ForegroundColor Green
        Write-Host ""
        Write-Host "Para usar Kody, ejecuta:" -ForegroundColor Cyan
        Write-Host "  kody --help" -ForegroundColor White
        Write-Host ""
        Write-Host "O desde el directorio:" -ForegroundColor Cyan
        Write-Host "  ~\kody\kody\target\release\kody.exe --help" -ForegroundColor White
        Write-Host ""
        Write-Host "[NOTA] Si 'kody' no funciona inmediatamente, cierra y abre" -ForegroundColor Yellow
        Write-Host "       PowerShell, o ejecuta: refreshenv" -ForegroundColor Yellow
        Write-Host ""
    }
    catch {
        Write-Host ""
        Write-Host "[ERROR] Error durante la instalacion: $_" -ForegroundColor Red
        exit 1
    }
}

Main