#!/bin/bash
# =============================================================================
# Kody - Script de Instalación para Linux/macOS
# =============================================================================
# Instalación con un solo comando:
#   curl -fsSL https://raw.githubusercontent.com/yokonad/kody/main/install.sh | bash
# =============================================================================

set -e

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
RESET='\033[0m'

# Banner
show_banner() {
    echo -e "${CYAN}"
    echo "+============================================================+"
    echo "|                       KODY                               |"
    echo "|              Scanner de Vulnerabilidades CLI              |"
    echo "+============================================================+"
    echo -e "${RESET}"
}

# Función para pausar
pause() {
    echo ""
    read -p "Presiona Enter para continuar..."
}

# Verificar si el comando existe
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# =============================================================================
# PASO 1: Instalar Rust
# =============================================================================
echo -e "[${MAGENTA}PASO 1${RESET}] Verificando Rust..."

if command_exists rustc && command_exists cargo; then
    RUST_VERSION=$(rustc --version | cut -d' ' -f2)
    echo -e "[${GREEN}OK${RESET}] Rust ya esta instalado: $RUST_VERSION"
else
    echo -e "[${YELLOW}INFO${RESET}] Rust no encontrado. Iniciando instalacion..."

    # Detectar SO
    if [[ "$OSTYPE" == "darwin"* ]]; then
        # macOS - verificar si brew esta instalado
        if command_exists brew; then
            echo -e "[${YELLOW}INFO${RESET}] Instalando Rust via Homebrew..."
            brew install rustup-init
            rustup-init
        else
            echo -e "[${YELLOW}INFO${RESET}] Instalando Rust via rustup..."
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        fi
    else
        # Linux
        echo -e "[${YELLOW}INFO${RESET}] Instalando Rust via rustup..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    fi

    # Cargar entorno de Rust
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi

    # Esperar y verificar
    sleep 2
    if command_exists rustc && command_exists cargo; then
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        echo -e "[${GREEN}OK${RESET}] Rust instalado correctamente: $RUST_VERSION"
    else
        echo -e "[${RED}ERROR${RESET}] Rust no se pudo instalar."
        echo -e "[${CYAN}INFO${RESET}] Por favor, instala Rust manualmente desde: https://rustup.rs"
        pause
        exit 1
    fi
fi

echo ""

# =============================================================================
# PASO 2: Clonar repositorio
# =============================================================================
echo -e "[${MAGENTA}PASO 2${RESET}] Descargando Kody..."

KODY_DIR="$HOME/kody"
PROJECT_DIR="$KODY_DIR/kody"

if [ -d "$PROJECT_DIR/.git" ]; then
    echo -e "[${YELLOW}INFO${RESET}] El repositorio ya existe. Actualizando..."
    cd "$PROJECT_DIR"
    git pull origin main 2>/dev/null || git pull origin master 2>/dev/null || true
else
    if [ -d "$KODY_DIR" ]; then
        echo -e "[${YELLOW}INFO${RESET}] Eliminando directorio anterior..."
        rm -rf "$KODY_DIR"
    fi

    echo -e "[${YELLOW}INFO${RESET}] Clonando repositorio..."
    git clone https://github.com/yokonad/kody.git "$KODY_DIR"
    cd "$PROJECT_DIR"
fi

echo -e "[${GREEN}OK${RESET}] Repositorio listo"
echo ""

# =============================================================================
# PASO 3: Compilar proyecto
# =============================================================================
echo -e "[${MAGENTA}PASO 3${RESET}] Compilando Kody..."
echo -e "[${YELLOW}INFO${RESET}] Este proceso puede tomar de 5 a 15 minutos..."
echo -e "[${YELLOW}INFO${RESET}] En la primera compilacion se descargan todas las dependencias."
echo ""

# Cargar entorno de Rust si es necesario
if [ -f "$HOME/.cargo/env" ]; then
    source "$HOME/.cargo/env"
fi

# Verificar que cargo esté disponible
if ! command_exists cargo; then
    echo -e "[${RED}ERROR${RESET}] Cargo no esta disponible en el PATH."
    echo -e "[${CYAN}INFO${RESET}] Ejecuta: source ~/.cargo/env"
    echo -e "[${CYAN}INFO${RESET}] O reinicia tu terminal."
    pause
    exit 1
fi

# Compilar
cargo build --release 2>&1 | tail -10

if [ -f "target/release/kody" ]; then
    echo ""
    echo -e "[${GREEN}OK${RESET}] Compilacion exitosa!"
else
    echo -e "[${RED}ERROR${RESET}] El archivo kody no se encontro despues de la compilacion."
    pause
    exit 1
fi

echo ""

# =============================================================================
# PASO 4: Instalar binario
# =============================================================================
echo -e "[${MAGENTA}PASO 4${RESET}] Instalando Kody..."

BIN_DIR="$HOME/.local/bin"
BIN_PATH="$BIN_DIR/kody"

# Crear directorio si no existe
mkdir -p "$BIN_DIR"

# Copiar binario
cp "target/release/kody" "$BIN_PATH"
chmod +x "$BIN_PATH"
echo -e "[${GREEN}OK${RESET}] Kody instalado en: $BIN_PATH"

# Agregar al PATH si es necesario
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    SHELL_RC="$HOME/.bashrc"
    [ -f "$HOME/.zshrc" ] && SHELL_RC="$HOME/.zshrc"
    [ -f "$HOME/.profile" ] && SHELL_RC="$HOME/.profile"

    if ! grep -q '.local/bin' "$SHELL_RC" 2>/dev/null; then
        echo '' >> "$SHELL_RC"
        echo '# Kody binary' >> "$SHELL_RC"
        echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
    fi
    export PATH="$BIN_DIR:$PATH"
    echo -e "[${GREEN}OK${RESET}] $BIN_DIR agregado al PATH"
fi

echo ""

# =============================================================================
# FINAL
# =============================================================================
echo -e "${GREEN}+============================================================+${RESET}"
echo -e "${GREEN}|         INSTALACION COMPLETADA EXITOSAMENTE!              |${RESET}"
echo -e "${GREEN}+============================================================+${RESET}"
echo ""
echo -e "Para usar Kody, cierra esta ventana y abre una Nueva terminal."
echo -e "Luego ejecuta:"
echo -e "  ${CYAN}kody --help${RESET}"
echo ""
echo -e "O desde el directorio:"
echo -e "  ${CYAN}$PROJECT_DIR/target/release/kody --help${RESET}"
echo ""
echo -e "${YELLOW}Nota:${RESET} Si 'kody' no funciona, ejecuta:"
echo -e "      ${CYAN}source ~/.bashrc${RESET}"
echo -e "      o reinicia tu terminal."
echo ""