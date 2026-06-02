# Kody - Script de Instalacion para Windows PowerShell
# Un solo comando: irm https://raw.githubusercontent.com/yokonad/kody/main/install.ps1 | iex

$host.UI.RawUI.WindowTitle = "Kody - Instalacion"

Write-Host ""
Write-Host "KODY - Scanner de Vulnerabilidades CLI" -ForegroundColor Cyan
Write-Host ""

$DebugMode = $true  # Cambiar a $false para produccion

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

function Write-Debug($msg) {
    if ($DebugMode) {
        Write-Host "  [DEBUG] $msg" -ForegroundColor DarkGray
    }
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
$ProjectDir = "$KodyDir\kody"

Write-Debug "KodyDir: $KodyDir"
Write-Debug "Git location: $((Get-Command git -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source))"
Write-Debug "Git version: $(git --version)"

# Limpiar instalacion anterior
if (Test-Path $KodyDir) {
    Write-Host "  [INFO] Limpiando instalacion anterior..." -ForegroundColor Cyan
    Write-Debug "Eliminando: $KodyDir"
    Remove-Item -Recurse -Force $KodyDir -ErrorAction SilentlyContinue
    Start-Sleep -Seconds 2

    if (Test-Path $KodyDir) {
        Write-Host "  [ERROR] No se pudo eliminar $KodyDir" -ForegroundColor Red
        Write-Debug "Todavia existe dopo Remove-Item"
        Pause-Script
        return
    }
    Write-Debug "Eliminacion exitosa"
}

Write-Host "  [INFO] Clonando repositorio..." -ForegroundColor Cyan
Write-Debug "Comando: git clone --depth 1 https://github.com/yokonad/kody.git $KodyDir"

# Clonar repositorio
$gitExitCode = 0
$gitOutput = ""

try {
    # Ejecutar git clone
    $gitResult = & git clone --depth 1 https://github.com/yokonad/kody.git $KodyDir 2>&1
    $gitExitCode = $LASTEXITCODE
    $gitOutput = $gitResult | Out-String

    Write-Debug "Git exit code: $gitExitCode"
    if ($gitOutput) {
        Write-Debug "Git output:"
        $gitOutput -split "`n" | Where-Object { $_.Trim() } | ForEach-Object {
            Write-Debug "  $_"
        }
    }
} catch {
    Write-Debug "Excepcion durante git clone: $_"
    $gitExitCode = 1
}

if ($gitExitCode -ne 0) {
    Write-Host "  [ERROR] Error al clonar repositorio (exit code: $gitExitCode)" -ForegroundColor Red
    Write-Debug "No se pudo clonar el repositorio"
    Write-Debug "Verifica tu conexion a internet"
    Write-Debug "URL: https://github.com/yokonad/kody"
    Pause-Script
    return
}

# Verificar que se descargo - git clone crea el directorio directamente
Write-Debug "Verificando descarga en: $KodyDir"

if (Test-Path $KodyDir) {
    $contents = Get-ChildItem $KodyDir -ErrorAction SilentlyContinue
    Write-Debug "Contenido de $KodyDir : $($contents.Count) items"

    if ($contents.Count -eq 0) {
        Write-Host "  [ERROR] El repositorio esta vacio." -ForegroundColor Red
        Pause-Script
        return
    }

    # El repositorio clonado esta en $KodyDir directamente (no en subdirectorio)
    Write-Debug "Repositorio clonado correctamente en $KodyDir"
} else {
    Write-Host "  [ERROR] El repositorio no se descargo." -ForegroundColor Red
    Write-Debug "$KodyDir no existe despues de git clone"
    Pause-Script
    return
}

Write-Host "  [OK] Repositorio listo" -ForegroundColor Green
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
    Write-Debug "cargo no encontrado en PATH"
    Pause-Script
    return
}

Write-Debug "Cargo disponible: $(cargo --version)"

try {
    Set-Location $KodyDir
    Write-Host "  [INFO] Compilando..." -ForegroundColor Cyan
    Write-Debug "Working directory: $(Get-Location)"

    cargo build --release 2>&1 | ForEach-Object { Write-Host "    $_" }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "  [ERROR] Compilacion fallo." -ForegroundColor Red
        Write-Debug "cargo build retorno exit code: $LASTEXITCODE"
        Pause-Script
        return
    }
} catch {
    Write-Host "  [ERROR] Error durante compilacion: $_" -ForegroundColor Red
    Pause-Script
    return
}

if (Test-Path "$KodyDir\target\release\kody.exe") {
    Write-Host "  [OK] Compilacion exitosa!" -ForegroundColor Green
    Write-Debug "Binario creado: $KodyDir\target\release\kody.exe"
} else {
    Write-Host "  [ERROR] kody.exe no encontrado." -ForegroundColor Red
    Write-Debug "No se encontro $KodyDir\target\release\kody.exe"
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

Copy-Item "$KodyDir\target\release\kody.exe" $BinPath -Force
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