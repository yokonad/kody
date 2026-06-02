# Kody - Script de Instalacion para Windows PowerShell
# Un solo comando: irm https://raw.githubusercontent.com/yokonad/kody/main/install.ps1 | iex

$host.UI.RawUI.WindowTitle = "Kody - Instalacion"

Write-Host ""
Write-Host "+============================================================+" -ForegroundColor Cyan
Write-Host "|                       KODY                               |" -ForegroundColor Red
Write-Host "|              Scanner de Vulnerabilidades CLI              |" -ForegroundColor Yellow
Write-Host "+============================================================+" -ForegroundColor Cyan
Write-Host ""
Write-Host "[INFO] Iniciando instalacion de Kody..." -ForegroundColor Cyan
Write-Host ""

function Test-Command($cmd) {
    $null = Get-Command $cmd -ErrorAction SilentlyContinue
    return $null -ne $null
}

function Test-RustInstalled {
    # Verificar rustc y cargo en varias ubicaciones
    $paths = @(
        "$env:USERPROFILE\.cargo\bin\rustc.exe",
        "$env:LOCALAPPDATA\Rustup\bin\rustc.exe",
        "$env:ProgramFiles\Rustup\bin\rustc.exe"
    )
    foreach ($p in $paths) {
        if (Test-Path $p) { return $true }
    }
    return (Test-Command rustc)
}

function Pause-Script {
    Write-Host ""
    Write-Host "Presiona ENTER para salir..." -ForegroundColor Gray
    $null = Read-Host
}

# PASO 1: Rust
Write-Host "[PASO 1] Verificando Rust..." -ForegroundColor Magenta

if (Test-RustInstalled) {
    $rustVersion = (rustc --version) -replace "rustc ", ""
    Write-Host "[OK] Rust instalado: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "[INFO] Rust no encontrado. Instalando..." -ForegroundColor Cyan
    Write-Host "[INFO]Descargando rustup..."

    $rustupUrl = "https://win.rustup.rs"
    $rustupPath = "$env:TEMP\rustup-init.exe"

    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -UseBasicParsing
    } catch {
        Write-Host "[ERROR] No se pudo descargar." -ForegroundColor Red
        Pause-Script
        return
    }

    Write-Host "[INFO] Instalando Rust automaticamente..."

    # Instalar rustup sin ventanas, modo silencioso
    # -y = yes to all
    # --default-toolchain stable = instalar toolchain estable
    # --profile minimal = instalacion minima sin extras
    $env:RUSTUP_HOME = "$env:USERPROFILE\.rustup"
    $env:CARGO_HOME = "$env:USERPROFILE\.cargo"
    $env:Path = "$env:CARGO_HOME\bin;$env:Path"

    & $rustupPath -y --default-toolchain stable --profile minimal --no-modify-path 2>&1 | Out-Null

    # Esperar instalacion
    Start-Sleep -Seconds 15

    # Refrescar variables de entorno
    $env:Path = "$env:CARGO_HOME\bin;" + [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "Machine")

    # Verificar final
    if (Test-RustInstalled) {
        $rustVersion = (rustc --version) -replace "rustc ", ""
        Write-Host "[OK] Rust instalado: $rustVersion" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] Rust no se instalo." -ForegroundColor Red
        Write-Host "[INFO] Instala manualmente desde: https://rustup.rs" -ForegroundColor Cyan
        Pause-Script
        return
    }
}

Write-Host ""

# PASO 2: Repo
Write-Host "[PASO 2] Descargando Kody..." -ForegroundColor Magenta
$KodyDir = "$HOME\kody"
$ProjectDir = "$KodyDir\kody"

if (Test-Path "$ProjectDir\.git") {
    Write-Host "[INFO] Actualizando repositorio..." -ForegroundColor Cyan
    Set-Location $ProjectDir
    git pull origin main 2>$null
} else {
    if (Test-Path $KodyDir) { Remove-Item -Recurse -Force $KodyDir }
    Write-Host "[INFO] Clonando repositorio..." -ForegroundColor Cyan
    git clone https://github.com/yokonad/kody.git $KodyDir 2>$null
}

if (Test-Path $ProjectDir) {
    Set-Location $ProjectDir
    Write-Host "[OK] Repositorio listo" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Error al descargar." -ForegroundColor Red
    Pause-Script
    return
}

Write-Host ""

# PASO 3: Compilar
Write-Host "[PASO 3] Compilando Kody..." -ForegroundColor Magenta
Write-Host "[INFO] Esto puede tomar 5-15 minutos..." -ForegroundColor Cyan

# Asegurar que cargo este en PATH
$env:CARGO_HOME = "$env:USERPROFILE\.cargo"
$env:RUSTUP_HOME = "$env:USERPROFILE\.rustup"
$env:Path = "$env:CARGO_HOME\bin;" + [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

if (-not (Test-Command cargo)) {
    Write-Host "[ERROR] Cargo no disponible." -ForegroundColor Red
    Write-Host "[INFO] Ejecuta en nueva terminal: rustup default stable" -ForegroundColor White
    Pause-Script
    return
}

try {
    Set-Location $ProjectDir
    Write-Host "[INFO] Compilando..." -ForegroundColor Cyan
    cargo build --release 2>$null
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Compilacion fallo." -ForegroundColor Red
        Pause-Script
        return
    }
} catch {
    Write-Host "[ERROR] Error: $_" -ForegroundColor Red
    Pause-Script
    return
}

if (Test-Path "target\release\kody.exe") {
    Write-Host "[OK] Compilacion exitosa!" -ForegroundColor Green
} else {
    Write-Host "[ERROR] kody.exe no encontrado." -ForegroundColor Red
    Pause-Script
    return
}

Write-Host ""

# PASO 4: Instalar
Write-Host "[PASO 4] Instalando..." -ForegroundColor Magenta
$BinDir = "$env:LOCALAPPDATA\bin\kody"
$BinPath = "$BinDir\kody.exe"

if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

Copy-Item "target\release\kody.exe" $BinPath -Force
Write-Host "[OK] Kody instalado en: $BinPath" -ForegroundColor Green

$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
    Write-Host "[OK] PATH actualizado" -ForegroundColor Green
}

Write-Host ""
Write-Host "+============================================================+" -ForegroundColor Green
Write-Host "|         INSTALACION COMPLETADA EXITOSAMENTE!              |" -ForegroundColor Green
Write-Host "+============================================================+" -ForegroundColor Green
Write-Host ""
Write-Host "Abre una NUEVA terminal PowerShell y ejecuta:" -ForegroundColor Cyan
Write-Host "  kody --help" -ForegroundColor White
Write-Host ""

Pause-Script