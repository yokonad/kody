#!/bin/bash
# Kody - Script de Instalacion para Linux/macOS
# Un solo comando: curl -fsSL https://raw.githubusercontent.com/yokonad/kody/main/install.sh | bash

# Colores
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
RESET='\033[0m'

echo ""
echo "KODY - Scanner de Vulnerabilidades CLI" -e "${CYAN}"
echo ""

# Verificar si Rust esta instalado
test_rust_installed() {
    if command -v rustc >/dev/null 2>&1; then
        return 0
    fi
    if [ -f "$HOME/.cargo/bin/rustc" ]; then
        return 0
    fi
    if [ -f "$HOME/.rustup/toolchains/stable/x86_64-unknown-linux-gnu/bin/rustc" ]; then
        return 0
    fi
    return 1
}

# Cargar entorno de Rust
load_rust_env() {
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
}

# PASO 1: Rust
echo -e "${MAGENTA}[PASO 1] Verificando Rust...${RESET}"

load_rust_env

if test_rust_installed; then
    RUST_VERSION=$(rustc --version 2>/dev/null | cut -d' ' -f2 || echo "desconocida")
    echo -e "  ${GREEN}[OK] Rust instalado: $RUST_VERSION${RESET}"
else
    echo -e "  ${CYAN}[INFO] Rust no encontrado. Instalando...${RESET}"

    if [[ "$OSTYPE" == "darwin"* ]]; then
        if command -v brew >/dev/null 2>&1; then
            brew install rustup-init
            rustup-init -y --default-toolchain stable --profile minimal
        else
            curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
        fi
    else
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable --profile minimal
    fi

    load_rust_env
    sleep 5

    if test_rust_installed; then
        RUST_VERSION=$(rustc --version | cut -d' ' -f2)
        echo -e "  ${GREEN}[OK] Rust instalado: $RUST_VERSION${RESET}"
    else
        echo -e "  ${RED}[ERROR] Rust no se instalo correctamente.${RESET}"
        echo -e "  ${CYAN}[INFO] Instala Rust manualmente desde: https://rustup.rs${RESET}"
        exit 1
    fi
fi

echo ""

# PASO 2: Repo
echo -e "${MAGENTA}[PASO 2] Descargando Kody...${RESET}"

KODY_DIR="$HOME/kody"
PROJECT_DIR="$KODY_DIR/kody"

if [ -d "$PROJECT_DIR/.git" ]; then
    echo -e "  ${CYAN}[INFO] Actualizando repositorio...${RESET}"
    cd "$PROJECT_DIR"
    git pull origin main 2>/dev/null || git pull origin master 2>/dev/null || true
else
    if [ -d "$KODY_DIR" ]; then
        rm -rf "$KODY_DIR"
    fi
    echo -e "  ${CYAN}[INFO] Clonando repositorio...${RESET}"
    git clone https://github.com/yokonad/kody.git "$KODY_DIR"
    cd "$PROJECT_DIR"
fi

if [ -d "$PROJECT_DIR" ]; then
    echo -e "  ${GREEN}[OK] Repositorio listo${RESET}"
else
    echo -e "  ${RED}[ERROR] Error al descargar repositorio.${RESET}"
    exit 1
fi

echo ""

# PASO 3: Compilar
echo -e "${MAGENTA}[PASO 3] Compilando Kody...${RESET}"
echo -e "  ${CYAN}[INFO] Esto puede tomar 5-15 minutos...${RESET}"

load_rust_env

if ! command -v cargo >/dev/null 2>&1; then
    echo -e "  ${RED}[ERROR] Cargo no disponible.${RESET}"
    echo -e "  ${CYAN}[INFO] Ejecuta: source ~/.cargo/env${RESET}"
    exit 1
fi

echo -e "  ${CYAN}[INFO] Compilando...${RESET}"
cargo build --release 2>&1 | tail -5

if [ -f "target/release/kody" ]; then
    echo -e "  ${GREEN}[OK] Compilacion exitosa!${RESET}"
else
    echo -e "  ${RED}[ERROR] Compilacion fallo.${RESET}"
    exit 1
fi

echo ""

# PASO 4: Instalar
echo -e "${MAGENTA}[PASO 4] Instalando...${RESET}"

BIN_DIR="$HOME/.local/bin"
BIN_PATH="$BIN_DIR/kody"

mkdir -p "$BIN_DIR"

cp "target/release/kody" "$BIN_PATH"
chmod +x "$BIN_PATH"
echo -e "  ${GREEN}[OK] Kody instalado en: $BIN_PATH${RESET}"

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
    echo -e "  ${GREEN}[OK] PATH actualizado${RESET}"
fi

echo ""
echo -e "${GREEN}INSTALACION COMPLETADA!${RESET}"
echo ""
echo -e "${CYAN}Abre una NUEVA terminal y ejecuta:${RESET}"
echo -e "  ${WHITE}kody --help${RESET}"
echo ""