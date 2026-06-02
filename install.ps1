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

function Pause-Script {
    Write-Host ""
    Write-Host "Presiona ENTER para salir..." -ForegroundColor Gray
    $null = Read-Host
}

# PASO 1: Rust
Write-Host "[PASO 1] Verificando Rust..." -ForegroundColor Magenta

if (Test-Command rustc) {
    $rustVersion = (rustc --version) -replace "rustc ", ""
    Write-Host "[OK] Rust instalado: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "[INFO] Rust no encontrado. Descargando rustup..." -ForegroundColor Cyan

    $rustupUrl = "https://win.rustup.rs"
    $rustupPath = "$env:TEMP\rustup-init.exe"

    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -UseBasicParsing
        Write-Host "[OK] rustup-init.exe descargado" -ForegroundColor Green
    } catch {
        Write-Host "[ERROR] No se pudo descargar rustup." -ForegroundColor Red
        Write-Host "[INFO] Descarga manualmente desde https://rustup.rs" -ForegroundColor Cyan
        Pause-Script
        return
    }

    Write-Host "[INFO] Instalando Rust (usa opcion 1 para instalacion default)..." -ForegroundColor Cyan
    Write-Host "[INFO] El instalador se abrira en una ventana separada." -ForegroundColor Yellow
    Write-Host ""

    # Ejecutar rustup con las opciones correctas
    # -y = yes to all
    # --default-toolchain stable = install stable as default
    # --profile minimal = minimal install
    $pinfo = New-Object System.Diagnostics.ProcessStartInfo
    $pinfo.FileName = $rustupPath
    $pinfo.Arguments = "-y --default-toolchain stable --profile minimal"
    $pinfo.UseShellExecute = $true
    $pinfo.WindowStyle = "Normal"
    $p = New-Object System.Diagnostics.Process
    $p.StartInfo = $pinfo
    $p.Start() | Out-Null
    $p.WaitForExit()

    $exitCode = $p.ExitCode
    Write-Host "[INFO] rustup finalizo con codigo: $exitCode" -ForegroundColor Cyan

    # Esperar un poco mas para que se complete la instalacion
    Start-Sleep -Seconds 10

    # Refrescar PATH
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    # Verificar
    Start-Sleep -Seconds 3

    if (Test-Command rustc) {
        $rustVersion = (rustc --version) -replace "rustc ", ""
        Write-Host "[OK] Rust instalado: $rustVersion" -ForegroundColor Green
    } else {
        Write-Host "[ERROR] Rust no se instalo correctamente." -ForegroundColor Red
        Write-Host "[INFO] Posibles soluciones:" -ForegroundColor Cyan
        Write-Host "  1. Ve a https://rustup.rs y descarga rustup-init.exe" -ForegroundColor White
        Write-Host "  2. Ejecútalo manualmente y elige opcion 1" -ForegroundColor White
        Write-Host "  3. Reinicia PowerShell y ejecuta este script de nuevo" -ForegroundColor White
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
    Write-Host "[INFO] Repositorio existe. Actualizando..." -ForegroundColor Cyan
    Set-Location $ProjectDir
    git pull origin main 2>$null
} else {
    if (Test-Path $KodyDir) {
        Remove-Item -Recurse -Force $KodyDir
    }
    Write-Host "[INFO] Clonando repositorio..." -ForegroundColor Cyan
    git clone https://github.com/yokonad/kody.git $KodyDir 2>$null
}

if (Test-Path $ProjectDir) {
    Set-Location $ProjectDir
    Write-Host "[OK] Repositorio listo" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Error al descargar repositorio." -ForegroundColor Red
    Pause-Script
    return
}

Write-Host ""

# PASO 3: Compilar
Write-Host "[PASO 3] Compilando Kody..." -ForegroundColor Magenta
Write-Host "[INFO] Esto puede tomar 5-15 minutos..." -ForegroundColor Cyan

$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

if (-not (Test-Command cargo)) {
    Write-Host "[ERROR] Cargo no disponible." -ForegroundColor Red
    Write-Host "[INFO] Ejecuta en una nueva terminal: rustup default stable" -ForegroundColor White
    Pause-Script
    return
}

try {
    Set-Location $ProjectDir
    Write-Host "[INFO] Compilando (puede tardar varios minutos)..." -ForegroundColor Cyan
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
Write-Host "Si no funciona, ejecuta: refreshenv" -ForegroundColor Yellow
Write-Host ""

Pause-Script