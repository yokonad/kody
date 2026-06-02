# =============================================================================
# Kody - Script de Instalación para Windows PowerShell
# =============================================================================
# Instalación con un solo comando:
#   irm https://raw.githubusercontent.com/yokonad/kody/main/install.ps1 | iex
# =============================================================================

$host.UI.RawUI.WindowTitle = "Kody - Instalacion"

Write-Host ""
Write-Host "+============================================================+" -ForegroundColor Cyan
Write-Host "|                       KODY                               |" -ForegroundColor Red
Write-Host "|              Scanner de Vulnerabilidades CLI              |" -ForegroundColor Yellow
Write-Host "+============================================================+" -ForegroundColor Cyan
Write-Host ""

Write-Host "[INFO] Iniciando instalacion de Kody..." -ForegroundColor Cyan
Write-Host ""

$global:ErrorOccurred = $false
$global:ErrorMessage = ""

# Verificar si el comando existe
function Test-Command($cmd) {
    try {
        $null = Get-Command $cmd -ErrorAction SilentlyContinue
        return $null -ne $null
    } catch {
        return $false
    }
}

# Función para pausar sin cerrar
function Pause-Script {
    Write-Host ""
    Write-Host "Presiona ENTER para salir..." -ForegroundColor Gray
    $null = Read-Host
}

# =============================================================================
# PASO 1: Instalar Rust
# =============================================================================
Write-Host "[PASO 1] Verificando Rust..." -ForegroundColor Magenta

if (Test-Command rustc) {
    $rustVersion = (rustc --version) -replace "rustc ", ""
    Write-Host "[OK] Rust ya esta instalado: $rustVersion" -ForegroundColor Green
} else {
    Write-Host "[PASO 1] Instalando Rust..." -ForegroundColor Magenta
    Write-Host "[INFO] Rust no encontrado. Iniciando descarga..." -ForegroundColor Cyan

    # Descargar rustup-init.exe
    $rustupUrl = "https://win.rustup.rs"
    $rustupPath = "$env:TEMP\rustup-init.exe"

    try {
        Write-Host "[INFO] Descargando rustup-init.exe..." -ForegroundColor Cyan
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -UseBasicParsing -TimeoutSec 60
    } catch {
        Write-Host ""
        Write-Host "[ERROR] No se pudo descargar rustup." -ForegroundColor Red
        Write-Host "[INFO] Descarga manualmente desde: https://rustup.rs" -ForegroundColor Cyan
        Pause-Script
        return
    }

    Write-Host "[INFO] Ejecutando instalador de Rust..." -ForegroundColor Cyan
    Write-Host "[INFO] ATENCION: Se abrira el instalador de Rust." -ForegroundColor Yellow
    Write-Host "[INFO] Sigue las instrucciones en pantalla." -ForegroundColor Yellow
    Write-Host ""

    try {
        $process = Start-Process -FilePath $rustupPath -ArgumentList "-y", "--default-toolchain", "stable" -PassThru -Wait
        if ($process.ExitCode -ne 0) {
            Write-Host "[WARN] El instalador pudo haber tenido problemas." -ForegroundColor Yellow
        }
    } catch {
        Write-Host ""
        Write-Host "[ERROR] Error al ejecutar el instalador: $_" -ForegroundColor Red
        Write-Host "[INFO]Descarga e instala Rust manualmente desde: https://rustup.rs" -ForegroundColor Cyan
        Pause-Script
        return
    }

    Write-Host "[INFO] Esperando finalizacion de instalacion..." -ForegroundColor Cyan
    Start-Sleep -Seconds 5

    # Refrescar entorno
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")
    Start-Sleep -Seconds 3

    if (Test-Command rustc) {
        $rustVersion = (rustc --version) -replace "rustc ", ""
        Write-Host "[OK] Rust instalado: $rustVersion" -ForegroundColor Green
    } else {
        Write-Host ""
        Write-Host "[ERROR] Rust no se pudo instalar correctamente." -ForegroundColor Red
        Write-Host "[INFO] Por favor, instala Rust manualmente:" -ForegroundColor Cyan
        Write-Host "1. Ve a: https://rustup.rs" -ForegroundColor White
        Write-Host "2. Descarga y ejecuta rustup-init.exe" -ForegroundColor White
        Write-Host "3. Elige la opcion '1' (default installation)" -ForegroundColor White
        Write-Host "4. Reinicia PowerShell y vuelve a ejecutar este script" -ForegroundColor White
        Pause-Script
        return
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
    Write-Host "[INFO] El repositorio ya existe. Actualizando..." -ForegroundColor Cyan
    try {
        Set-Location $ProjectDir
        git pull origin main 2>$null
    } catch {
        Write-Host "[WARN] No se pudo actualizar. Descargando repositorio fresco..." -ForegroundColor Yellow
        Remove-Item -Recurse -Force $KodyDir -ErrorAction SilentlyContinue
        git clone https://github.com/yokonad/kody.git $KodyDir 2>$null
    }
} else {
    if (Test-Path $KodyDir) {
        Write-Host "[INFO] Eliminando instalacion anterior..." -ForegroundColor Cyan
        Remove-Item -Recurse -Force $KodyDir -ErrorAction SilentlyContinue
    }

    Write-Host "[INFO] Clonando repositorio..." -ForegroundColor Cyan
    try {
        git clone https://github.com/yokonad/kody.git $KodyDir 2>$null
    } catch {
        Write-Host ""
        Write-Host "[ERROR] No se pudo clonar el repositorio." -ForegroundColor Red
        Write-Host "[INFO] Verifica tu conexion a internet." -ForegroundColor Cyan
        Pause-Script
        return
    }
}

if (Test-Path $ProjectDir) {
    Set-Location $ProjectDir
    Write-Host "[OK] Repositorio listo" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[ERROR] El repositorio no se descargo correctamente." -ForegroundColor Red
    Pause-Script
    return
}

Write-Host ""

# =============================================================================
# PASO 3: Compilar proyecto
# =============================================================================
Write-Host "[PASO 3] Compilando Kody..." -ForegroundColor Magenta
Write-Host "[INFO] Este proceso puede tomar de 5 a 15 minutos..." -ForegroundColor Cyan
Write-Host "[INFO] En la primera compilacion se descargan las dependencias." -ForegroundColor Gray
Write-Host ""

# Refrescar PATH
$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

# Verificar cargo
if (-not (Test-Command cargo)) {
    Write-Host ""
    Write-Host "[ERROR] Cargo no esta disponible en el PATH." -ForegroundColor Red
    Write-Host "[INFO] Cierra esta ventana y abre PowerShell nuevo." -ForegroundColor Cyan
    Write-Host "[INFO] Luego ejecuta: rustup default stable" -ForegroundColor White
    Write-Host "[INFO] Y vuelve a ejecutar este script." -ForegroundColor White
    Pause-Script
    return
}

try {
    Set-Location $ProjectDir
    Write-Host "[INFO] Compilando... espera por favor..." -ForegroundColor Cyan

    $cargoOutput = cargo build --release 2>&1
    $lastLines = $cargoOutput | Select-Object -Last 10
    foreach ($line in $lastLines) {
        Write-Host "    $line" -ForegroundColor DarkGray
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "[ERROR] La compilacion fallo." -ForegroundColor Red
        Write-Host "[INFO] Revisa los mensajes de error arriba." -ForegroundColor Cyan
        Pause-Script
        return
    }
} catch {
    Write-Host ""
    Write-Host "[ERROR] Error durante la compilacion: $_" -ForegroundColor Red
    Pause-Script
    return
}

if (Test-Path "target\release\kody.exe") {
    Write-Host "[OK] Compilacion exitosa!" -ForegroundColor Green
} else {
    Write-Host ""
    Write-Host "[ERROR] kody.exe no se encontro despues de la compilacion." -ForegroundColor Red
    Pause-Script
    return
}

Write-Host ""

# =============================================================================
# PASO 4: Instalar binario
# =============================================================================
Write-Host "[PASO 4] Instalando Kody..." -ForegroundColor Magenta

$BinDir = "$env:LOCALAPPDATA\bin\kody"
$BinPath = "$BinDir\kody.exe"

# Crear directorio
try {
    if (-not (Test-Path $BinDir)) {
        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    }
} catch {
    Write-Host "[WARN] No se pudo crear el directorio $BinDir" -ForegroundColor Yellow
}

# Copiar binario
try {
    Copy-Item "target\release\kody.exe" $BinPath -Force
    Write-Host "[OK] Kody instalado en: $BinPath" -ForegroundColor Green
} catch {
    Write-Host ""
    Write-Host "[ERROR] No se pudo copiar el archivo: $_" -ForegroundColor Red
    Write-Host "[INFO] Puedes ejecutar Kody desde:" -ForegroundColor Cyan
    Write-Host "  $ProjectDir\target\release\kody.exe" -ForegroundColor White
    Pause-Script
    return
}

# Agregar al PATH
$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    try {
        [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
        Write-Host "[OK] $BinDir agregado al PATH" -ForegroundColor Green
    } catch {
        Write-Host "[WARN] No se pudo agregar al PATH automaticamente." -ForegroundColor Yellow
    }
}

Write-Host ""

# =============================================================================
# FINAL
# =============================================================================
Write-Host "+============================================================+" -ForegroundColor Green
Write-Host "|         INSTALACION COMPLETADA EXITOSAMENTE!              |" -ForegroundColor Green
Write-Host "+============================================================+" -ForegroundColor Green
Write-Host ""
Write-Host "Cierra esta ventana y abre una Nueva terminal PowerShell." -ForegroundColor Cyan
Write-Host "Luego ejecuta uno de estos comandos:" -ForegroundColor Cyan
Write-Host ""
Write-Host "  kody --help" -ForegroundColor White
Write-Host "  kody scan 192.168.1.1 --ports 1-1024" -ForegroundColor White
Write-Host "  kody auto-scan" -ForegroundColor White
Write-Host ""
Write-Host "Si 'kody' no funciona en la nueva terminal, ejecuta:" -ForegroundColor Yellow
Write-Host "  refreshenv" -ForegroundColor White
Write-Host "O reinicia tu computadora." -ForegroundColor Yellow
Write-Host ""

Pause-Script