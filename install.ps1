# =============================================================================
# Kody - Script de Instalación para Windows PowerShell
# =============================================================================
# Instalación con un solo comando:
#   irm https://raw.githubusercontent.com/yokonad/kody/main/install.ps1 | iex
# =============================================================================

$host.UI.RawUI.WindowTitle = "Kody - Instalación"

Write-Host ""
Write-Host "+============================================================+" -ForegroundColor Cyan
Write-Host "|                       KODY                               |" -ForegroundColor Red
Write-Host "|              Scanner de Vulnerabilidades CLI              |" -ForegroundColor Yellow
Write-Host "+============================================================+" -ForegroundColor Cyan
Write-Host ""

$global:ErrorOccurred = $false
$global:ErrorMessage = ""

# Función para mostrar errores SIN cerrar
function Show-Error($msg) {
    $host.UI.Write-HostLine("")
    $host.UI.Write-HostLine("============================================================" -ForegroundColor Red)
    $host.UI.Write-HostLine("[ERROR] $msg" -ForegroundColor Red)
    $host.UI.Write-HostLine("============================================================" -ForegroundColor Red)
    $host.UI.Write-HostLine("")
    $global:ErrorOccurred = $true
    $global:ErrorMessage = $msg
}

# Función para mostrar información
function Show-Info($msg) {
    $host.UI.Write-HostLine("[INFO] $msg" -ForegroundColor Cyan)
}

# Función para mostrar éxito
function Show-Success($msg) {
    $host.UI.Write-HostLine("[OK] $msg" -ForegroundColor Green)
}

# Función para mostrar advertencia
function Show-Warning($msg) {
    $host.UI.Write-HostLine("[WARN] $msg" -ForegroundColor Yellow)
}

# Función para pausar
function Pause-Script {
    Write-Host ""
    Write-Host "Presiona cualquier tecla para salir..." -ForegroundColor Gray
    $null = $host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
}

# Verificar si el comando existe
function Test-Command($cmd) {
    try {
        $null = Get-Command $cmd -ErrorAction SilentlyContinue
        return $null -ne $null
    } catch {
        return $false
    }
}

Write-Info "Iniciando instalacion de Kody..."
Write-Host ""

# =============================================================================
# PASO 1: Instalar Rust
# =============================================================================
Write-Host "[PASO 1] Verificando Rust..." -ForegroundColor Magenta

if (Test-Command rustc) {
    $rustVersion = (rustc --version) -replace "rustc ", ""
    Show-Success "Rust ya esta instalado: $rustVersion"
} else {
    Write-Host "[PASO 1] Instalando Rust..." -ForegroundColor Magenta
    Show-Info "Rust no encontrado. Iniciando descarga..."

    # Descargar rustup-init.exe
    $rustupUrl = "https://win.rustup.rs"
    $rustupPath = "$env:TEMP\rustup-init.exe"

    try {
        Show-Info "Descargando rustup-init.exe..."
        [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
        Invoke-WebRequest -Uri $rustupUrl -OutFile $rustupPath -UseBasicParsing -TimeoutSec 60
    } catch {
        Show-Error "No se pudo descargar rustup. Verifica tu conexión a internet."
        Show-Info "Descarga manualmente desde: https://rustup.rs"
        Pause-Script
        return
    }

    Show-Info "Ejecutando instalador de Rust..."
    Show-Info "ATENCION: Se abrira una ventana del instalador de Rust."
    Show-Info "Sigue las instrucciones en pantalla (usa la opcion '1' para instalar predeterminada)."
    Write-Host ""

    try {
        # Ejecutar rustup y esperar
        $process = Start-Process -FilePath $rustupPath -ArgumentList "-y", "--default-toolchain", "stable" -PassThru -Wait

        if ($process.ExitCode -ne 0) {
            Show-Warning "El instalador de Rust pudo haber tenido problemas."
        }
    } catch {
        Show-Error "Error al ejecutar el instalador de Rust: $_"
        Show-Info "Descarga e instala Rust manualmente desde: https://rustup.rs"
        Pause-Script
        return
    }

    # Esperar a que se complete la instalación
    Show-Info "Esperando finalizacion de instalacion..."
    Start-Sleep -Seconds 5

    # Refrescar entorno
    $env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

    # Esperar y verificar instalación
    Start-Sleep -Seconds 3

    if (Test-Command rustc) {
        $rustVersion = (rustc --version) -replace "rustc ", ""
        Show-Success "Rust instalado: $rustVersion"
    } else {
        Show-Error "Rust no se pudo instalar correctamente."
        Show-Info "Por favor, instala Rust manualmente:"
        Show-Info "1. Ve a: https://rustup.rs"
        Show-Info "2. Descarga y ejecuta rustup-init.exe"
        Show-Info "3. Elige la opcion '1) Proceed with default installation'"
        Show-Info "4. Reinicia PowerShell y vuelve a ejecutar este script"
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
    Show-Info "El repositorio ya existe. Actualizando..."
    try {
        Set-Location $ProjectDir
        git pull origin main 2>$null
    } catch {
        # Si falla el pull, eliminar y clonar de nuevo
        Show-Warning "No se pudo actualizar. Descargando repositorio fresco..."
        Remove-Item -Recurse -Force $KodyDir -ErrorAction SilentlyContinue
        git clone https://github.com/yokonad/kody.git $KodyDir 2>$null
    }
} else {
    if (Test-Path $KodyDir) {
        Show-Info "Eliminando instalacion anterior..."
        Remove-Item -Recurse -Force $KodyDir -ErrorAction SilentlyContinue
    }

    Show-Info "Clonando repositorio..."
    try {
        git clone https://github.com/yokonad/kody.git $KodyDir 2>$null
    } catch {
        Show-Error "No se pudo clonar el repositorio. Verifica tu conexion a internet."
        Pause-Script
        return
    }
}

if (Test-Path $ProjectDir) {
    Set-Location $ProjectDir
    Show-Success "Repositorio listo"
} else {
    Show-Error "El repositorio no se descargo correctamente."
    Pause-Script
    return
}

Write-Host ""

# =============================================================================
# PASO 3: Compilar proyecto
# =============================================================================
Write-Host "[PASO 3] Compilando Kody..." -ForegroundColor Magenta
Show-Info "Este proceso puede tomar de 5 a 15 minutos..."
Show-Info "En la primera compilacion se descargan las dependencias de Rust."

# Refrescar PATH
$env:Path = [System.Environment]::GetEnvironmentVariable("Path", "Machine") + ";" + [System.Environment]::GetEnvironmentVariable("Path", "User")

# Verificar que cargo esté disponible
if (-not (Test-Command cargo)) {
    Show-Error "Cargo no esta disponible en el PATH."
    Show-Info "Cierra esta ventana, abre PowerShell nuevo y ejecuta:"
    Show-Info "  rustup default stable"
    Show-Info "Luego vuelve a ejecutar este script."
    Pause-Script
    return
}

try {
    Set-Location $ProjectDir

    Show-Info "Compilando... esto puede tardar varios minutos..."

    # Compilar y mostrar progreso
    $cargoOutput = cargo build --release 2>&1
    $lastLines = $cargoOutput | Select-Object -Last 10
    foreach ($line in $lastLines) {
        Write-Host "    $line" -ForegroundColor DarkGray
    }

    if ($LASTEXITCODE -ne 0) {
        Show-Error "La compilacion fallo. Codigo de error: $LASTEXITCODE"
        Show-Info "Revisa los mensajes de error arriba."
        Pause-Script
        return
    }
} catch {
    Show-Error "Error durante la compilacion: $_"
    Pause-Script
    return
}

if (Test-Path "target\release\kody.exe") {
    Show-Success "Compilacion exitosa!"
} else {
    Show-Error "El archivo kody.exe no se encontro despues de la compilacion."
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
    Show-Warning "No se pudo crear el directorio $BinDir"
}

# Copiar binario
try {
    Copy-Item "target\release\kody.exe" $BinPath -Force
    Show-Success "Kody instalado en: $BinPath"
} catch {
    Show-Error "No se pudo copiar el archivo: $_"
    Show-Info "Puedes ejecutar Kody directamente desde:"
    Show-Info "$ProjectDir\target\release\kody.exe"
    Pause-Script
    return
}

# Agregar al PATH
$UserPath = [System.Environment]::GetEnvironmentVariable("Path", "User")
if ($UserPath -notlike "*$BinDir*") {
    try {
        [System.Environment]::SetEnvironmentVariable("Path", "$UserPath;$BinDir", "User")
        Show-Success "$BinDir agregado al PATH del usuario"
    } catch {
        Show-Warning "No se pudo agregar al PATH automaticamente."
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