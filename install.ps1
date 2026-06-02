# Kody - Script de Instalacion para Windows PowerShell
# Un solo comando: irm https://raw.githubusercontent.com/yokonad/kody/main/install.ps1 | iex

$host.UI.RawUI.WindowTitle = "Kody - Instalacion"

Write-Host ""
Write-Host "KODY - Scanner de Vulnerabilidades CLI" -ForegroundColor Cyan
Write-Host ""

function Test-Command($cmd) {
    $null = Get-Command $cmd -ErrorAction SilentlyContinue
    return $null -ne $null
}

function Test-CommandInPath($cmd) {
    if (Test-Command $cmd) { return $true }

    $gitPaths = @(
        "C:\Program Files\Git\cmd",
        "C:\Program Files (x86)\Git\cmd",
        "$env:ProgramFiles\Git\cmd",
        "$env:LOCALAPPDATA\Programs\Git\cmd"
    )

    foreach ($gitPath in $gitPaths) {
        $gitExe = Join-Path $gitPath "$cmd.exe"
        if (Test-Path $gitExe) {
            if ($env:Path -notlike "*$gitPath*") {
                $env:Path = "$gitPath;$env:Path"
            }
            return $true
        }
    }

    return $false
}

function Test-RustInstalled {
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

# =============================================================================
# PRE-INSTALACION
# =============================================================================
Write-Host "[PRE-INSTALACION] Verificando dependencias..." -ForegroundColor Magenta

$needsGit = -not (Test-CommandInPath git)
$needsRust = -not (Test-RustInstalled)

if ($needsGit) {
    Write-Host "  [INFO] Git no encontrado. Se instalara." -ForegroundColor Cyan
}

if ($needsRust) {
    Write-Host "  [INFO] Rust no encontrado. Se instalara." -ForegroundColor Cyan
}

if (-not $needsGit -and -not $needsRust) {
    Write-Host "  [INFO] Git y Rust ya instalados." -ForegroundColor Cyan
}

Write-Host ""

# Instalar Git
if ($needsGit) {
    Write-Host "[PASO 1] Instalando Git..." -ForegroundColor Magenta

    $wingetAvailable = Test-Command winget

    if ($wingetAvailable) {
        Write-Host "  [INFO] Usando winget..." -ForegroundColor Cyan
        winget install --id Git.Git --exact --silent --accept-package-agreements --accept-source-agreements 2>&1 | Out-Null
        Start-Sleep -Seconds 5
    } else {
        Write-Host "  [INFO] Descargando instalador de Git..." -ForegroundColor Cyan
        $gitInstallerUrl = "https://github.com/git-for-windows/git/releases/download/v2.47.0.windows.1/Git-2.47.0-64-bit.exe"
        $gitInstallerPath = "$env:TEMP\git-installer.exe"

        try {
            [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
            Invoke-WebRequest -Uri $gitInstallerUrl -OutFile $gitInstallerPath -UseBasicParsing -TimeoutSec 120
        } catch {
            Write-Host "  [ERROR] No se pudo descargar Git." -ForegroundColor Red
            Pause-Script
            return
        }

        Write-Host "  [INFO] Ejecutando instalador..." -ForegroundColor Cyan
        Start-Process -FilePath $gitInstallerPath -ArgumentList "/VERYSILENT", "/NORESTART", "/NOCANCEL", "/SP-" -Wait -NoNewWindow
        Remove-Item $gitInstallerPath -Force -ErrorAction SilentlyContinue
        Start-Sleep -Seconds 5
    }

    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    if (Test-CommandInPath git) {
        Write-Host "  [OK] Git instalado: $((git --version) -replace 'git version ')" -ForegroundColor Green
    } else {
        Write-Host "  [ERROR] Git no se instalo." -ForegroundColor Red
        Write-Host "  [INFO] Instala Git manualmente desde: https://git-scm.com/download/win" -ForegroundColor Cyan
        Pause-Script
        return
    }
} else {
    Write-Host "[PASO 1] Git..." -ForegroundColor Magenta
    Write-Host "  [OK] Git ya instalado: $((git --version) -replace 'git version ')" -ForegroundColor Green
}

Write-Host ""

# Instalar Rust
if ($needsRust) {
    Write-Host "[PASO 2] Instalando Rust..." -ForegroundColor Magenta
    Write-Host "  [INFO] Descargando rustup..." -ForegroundColor Cyan

    $rustupUrl = "https://win.rustup.rs"
    $rustupPath = "$env:TEMP\rustup-init.exe"

    try {
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -UseBasicParsing
    } catch {
        Write-Host "  [ERROR] No se pudo descargar Rust." -ForegroundColor Red
        Write-Host "  [INFO] Instala Rust manualmente desde: https://rustup.rs" -ForegroundColor Cyan
        Pause-Script
        return
    }

    Write-Host "  [INFO] Instalando Rust..." -ForegroundColor Cyan

    $env:RUSTUP_HOME = "$env:USERPROFILE\.rustup"
    $env:CARGO_HOME = "$env:USERPROFILE\.cargo"
    $env:Path = "$env:CARGO_HOME\bin;$env:Path"

    & $rustupPath -y --default-toolchain stable --profile minimal --no-modify-path 2>&1 | Out-Null

    Start-Sleep -Seconds 15

    $env:Path = "$env:CARGO_HOME\bin;" + [System.Environment]::GetEnvironmentVariable("Path", "User") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "Machine")

    if (Test-RustInstalled) {
        Write-Host "  [OK] Rust instalado: $((rustc --version) -replace 'rustc ')" -ForegroundColor Green
    } else {
        Write-Host "  [ERROR] Rust no se instalo." -ForegroundColor Red
        Write-Host "  [INFO] Instala Rust manualmente desde: https://rustup.rs" -ForegroundColor Cyan
        Pause-Script
        return
    }
} else {
    Write-Host "[PASO 2] Rust..." -ForegroundColor Magenta
    Write-Host "  [OK] Rust ya instalado: $((rustc --version) -replace 'rustc ')" -ForegroundColor Green
}

Write-Host ""

# =============================================================================
# PASO 3: Repo
# =============================================================================
Write-Host "[PASO 3] Descargando Kody..." -ForegroundColor Magenta

$KodyDir = "$HOME\kody"

Write-Host "  [INFO] Directorio destino: $KodyDir" -ForegroundColor Cyan
Write-Host "  [INFO] Git: $((Get-Command git -ErrorAction SilentlyContinue).Source)" -ForegroundColor Cyan
Write-Host "  [INFO] Git version: $(git --version)" -ForegroundColor Cyan

# Limpiar instalacion anterior
if (Test-Path $KodyDir) {
    Write-Host "  [INFO] Limpiando instalacion anterior..." -ForegroundColor Cyan
    Remove-Item -Recurse -Force $KodyDir -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 3

    if (Test-Path $KodyDir) {
        Write-Host "  [WARN] No se pudo eliminar. Reintentando..." -ForegroundColor Yellow
        Start-Sleep -Seconds 2
        Remove-Item -Recurse -Force $KodyDir -ErrorAction SilentlyContinue
    }

    if (Test-Path $KodyDir) {
        Write-Host "  [ERROR] No se pudo eliminar $KodyDir" -ForegroundColor Red
        Write-Host "  [INFO] Cierra todos los programas que puedan usar esa carpeta." -ForegroundColor Cyan
        Write-Host "  [INFO] Luego intenta de nuevo manualmente." -ForegroundColor Cyan
        Pause-Script
        return
    }
    Write-Host "  [OK] Limpiado correctamente" -ForegroundColor Green
}

Write-Host "  [INFO] Ejecutando: git clone --depth 1 https://github.com/yokonad/kody.git" -ForegroundColor Cyan

# Clonar repositorio - mostrar todo el output
$gitOutput = ""
$gitError = $null

try {
    $gitResult = & git clone --depth 1 https://github.com/yokonad/kody.git $KodyDir 2>&1
    $gitExitCode = $LASTEXITCODE

    # Convertir output a string
    $gitOutput = $gitResult | Out-String

    Write-Host "  [DEBUG] Git exit code: $gitExitCode" -ForegroundColor DarkCyan

    if ($gitOutput -and $gitOutput.Trim()) {
        Write-Host "  [DEBUG] Git output:" -ForegroundColor DarkCyan
        $gitOutput.Trim() -split "`n" | ForEach-Object {
            Write-Host "    $_" -ForegroundColor DarkCyan
        }
    }

    if ($gitExitCode -ne 0) {
        Write-Host "  [ERROR] Git clone fallido (exit code: $gitExitCode)" -ForegroundColor Red
        Write-Host "  [INFO] Posibles causas:" -ForegroundColor Yellow
        Write-Host "    - Sin conexion a internet" -ForegroundColor White
        Write-Host "    - Proxy o firewall bloqueando" -ForegroundColor White
        Write-Host "    - URL incorrecta" -ForegroundColor White
        Pause-Script
        return
    }

} catch {
    Write-Host "  [ERROR] Excepcion al clonar: $_" -ForegroundColor Red
    Pause-Script
    return
}

# Verificar descarga
if (-not (Test-Path $KodyDir)) {
    Write-Host "  [ERROR] El directorio $KodyDir no existe despues de git clone" -ForegroundColor Red
    Write-Host "  [DEBUG] gitOutput fue: $gitOutput" -ForegroundColor DarkCyan
    Pause-Script
    return
}

$contents = Get-ChildItem $KodyDir -ErrorAction SilentlyContinue
if ($null -eq $contents -or $contents.Count -eq 0) {
    Write-Host "  [ERROR] El repositorio esta vacio" -ForegroundColor Red
    Pause-Script
    return
}

Write-Host "  [OK] Descargado! Archivos: $($contents.Count)" -ForegroundColor Green
Write-Host ""

# =============================================================================
# PASO 4: Compilar
# =============================================================================
Write-Host "[PASO 4] Compilando Kody..." -ForegroundColor Magenta
Write-Host "  [INFO] Esto puede tomar 5-15 minutos..." -ForegroundColor Cyan

$env:CARGO_HOME = "$env:USERPROFILE\.cargo"
$env:RUSTUP_HOME = "$env:USERPROFILE\.rustup"
$env:Path = "$env:CARGO_HOME\bin;" + [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

if (-not (Test-Command cargo)) {
    Write-Host "  [ERROR] Cargo no disponible." -ForegroundColor Red
    Pause-Script
    return
}

try {
    Set-Location $KodyDir
    Write-Host "  [INFO] Working dir: $(Get-Location)" -ForegroundColor Cyan
    Write-Host "  [INFO] Ejecutando: cargo build --release" -ForegroundColor Cyan

    $cargoOutput = cargo build --release 2>&1 | Out-String
    Write-Host "  [DEBUG] Cargo output:" -ForegroundColor DarkCyan
    $cargoOutput -split "`n" | Select-Object -First 5 | ForEach-Object {
        Write-Host "    $_" -ForegroundColor DarkCyan
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [ERROR] Compilacion fallo (exit code: $LASTEXITCODE)" -ForegroundColor Red
        Write-Host "  [DEBUG] Full cargo output:" -ForegroundColor DarkCyan
        $cargoOutput -split "`n" | ForEach-Object { Write-Host "    $_" -ForegroundColor DarkCyan }
        Pause-Script
        return
    }
} catch {
    Write-Host "  [ERROR] Error durante compilacion: $_" -ForegroundColor Red
    Pause-Script
    return
}

$exePath = "$KodyDir\target\release\kody.exe"
if (Test-Path $exePath) {
    Write-Host "  [OK] Compilacion exitosa!" -ForegroundColor Green
    $exeSize = (Get-Item $exePath).Length / 1MB
    Write-Host "  [INFO] Tamano: $([math]::Round($exeSize, 2)) MB" -ForegroundColor Cyan
} else {
    Write-Host "  [ERROR] kody.exe no encontrado en: $exePath" -ForegroundColor Red
    Pause-Script
    return
}

Write-Host ""

# =============================================================================
# PASO 5: Instalar
# =============================================================================
Write-Host "[PASO 5] Instalando..." -ForegroundColor Magenta
$BinDir = "$env:LOCALAPPDATA\bin\kody"
$BinPath = "$BinDir\kody.exe"

if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

Copy-Item $exePath $BinPath -Force
Write-Host "  [OK] Kody instalado en: $BinPath" -ForegroundColor Green

$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
    Write-Host "  [OK] PATH actualizado" -ForegroundColor Green
}

Write-Host ""
Write-Host "INSTALACION COMPLETADA!" -ForegroundColor Green
Write-Host ""
Write-Host "Abre una NUEVA terminal PowerShell y ejecuta:" -ForegroundColor Cyan
Write-Host "  kody --help" -ForegroundColor White
Write-Host ""

Pause-Script