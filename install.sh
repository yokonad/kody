#!/bin/bash
# =============================================================================
# Kody - Script de Instalación
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
CYAN='\033[0;36m'
RESET='\033[0m'

# Banner
show_banner() {
    echo -e "${CYAN}"
    echo "╔══════════════════════════════════════════════════════════════════╗"
    echo "║                                                                  ║"
    echo "║                            ${RED}KODY${CYAN}                           ║"
    echo "║                                                                  ║"
    echo "║                ${YELLOW}Scanner de Vulnerabilidades CLI${CYAN}              ║"
    echo "║                      ${YELLOW}Desarrollado en Rust${CYAN}                   ║"
    echo "║                                                                  ║"
    echo "╚══════════════════════════════════════════════════════════════════╝"
    echo -e "${RESET}"
}

# Verificar si el comando existe
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Verificar e instalar Rust
install_rust() {
    if command_exists rustc && command_exists cargo; then
        echo -e "${GREEN}✓${RESET} Rust ya está instalado: $(rustc --version | cut -d' ' -f2)"
        return 0
    fi

    echo -e "${YELLOW}▸${RESET} Rust no encontrado. Instalando Rust..."

    # Instalar rustup
    if command_exists curl; then
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    elif command_exists wget; then
        wget -qO- https://sh.rustup.rs | sh -s -- -y
    else
        echo -e "${RED}✗${RESET} Necesitas curl o wget para instalar Rust."
        echo "   Instala curl/wget primero, o instala Rust manualmente desde https://rustup.rs"
        exit 1
    fi

    # Cargar entorno de Rust
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi

    # Verificar instalación
    if command_exists rustc && command_exists cargo; then
        echo -e "${GREEN}✓${RESET} Rust instalado correctamente: $(rustc --version | cut -d' ' -f2)"
    else
        echo -e "${RED}✗${RESET} Error al instalar Rust."
        echo "   Por favor, instala Rust manualmente desde https://rustup.rs"
        exit 1
    fi
}

# Clonar o actualizar repositorio
setup_repo() {
    KODY_DIR="$HOME/kody"

    if [ -d "$KODY_DIR/.git" ]; then
        echo -e "${YELLOW}▸${RESET} Actualizando repositorio existente..."
        cd "$KODY_DIR"
        git pull origin main 2>/dev/null || git pull origin master 2>/dev/null || true
    else
        echo -e "${YELLOW}▸${RESET} Clonando repositorio..."
        git clone https://github.com/yokonad/kody.git "$KODY_DIR"
        cd "$KODY_DIR"
    fi

    echo -e "${GREEN}✓${RESET} Repositorio listo en: $KODY_DIR"
}

# Compilar proyecto
build_project() {
    echo -e "${YELLOW}▸${RESET} Compilando proyecto (esto puede tomar unos minutos)..."

    # Cargar entorno de Rust si es necesario
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi

    cd kody

    # Compilar con optimizations
    cargo build --release 2>&1 | tail -5

    if [ -f "target/release/kody" ]; then
        echo -e "${GREEN}✓${RESET} Compilación exitosa!"
    else
        echo -e "${RED}✗${RESET} Error en la compilación."
        exit 1
    fi
}

# Crear enlace simbólico global
install_binary() {
    BIN_PATH="$HOME/.local/bin/kody"

    # Crear directorio si no existe
    mkdir -p "$HOME/.local/bin"

    # Copiar binario
    cp "target/release/kody" "$BIN_PATH"
    chmod +x "$BIN_PATH"

    # Agregar al PATH si es necesario
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        SHELL_RC="$HOME/.bashrc"
        [ -f "$HOME/.zshrc" ] && SHELL_RC="$HOME/.zshrc"

        if ! grep -q '.local/bin' "$SHELL_RC" 2>/dev/null; then
            echo '' >> "$SHELL_RC"
            echo '# Kody binary' >> "$SHELL_RC"
            echo 'export PATH="$HOME/.local/bin:$PATH"' >> "$SHELL_RC"
        fi
        export PATH="$HOME/.local/bin:$PATH"
    fi

    echo -e "${GREEN}✓${RESET} Kody instalado en: $BIN_PATH"
}

# Función principal
main() {
    show_banner

    echo -e "${BLUE}▸${RESET} Iniciando instalación de Kody..."
    echo ""

    install_rust
    setup_repo
    build_project
    install_binary

    echo ""
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════════${RESET}"
    echo -e "${GREEN}  ✓${RESET} ¡Instalación completada!${GREEN}                                            ║${RESET}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════════════${RESET}"
    echo ""
    echo -e "Para usar Kody, ejecuta:"
    echo -e "  ${CYAN}kody --help${RESET}"
    echo ""
    echo -e "O desde el directorio:"
    echo -e "  ${CYAN}~/kody/kody/target/release/kody --help${RESET}"
    echo ""
    echo -e "${YELLOW}Nota:${RESET} Si 'kody' no funciona inmediatamente, reinicia tu terminal"
    echo "      o ejecuta: ${CYAN}source ~/.bashrc${RESET}"
    echo ""
}

# Ejecutar
main "$@"