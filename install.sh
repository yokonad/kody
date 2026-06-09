#!/bin/bash
# Kody - Instalador para Linux/macOS (binario pre-compilado, ~10 segundos)
# Un solo comando: curl -fsSL https://raw.githubusercontent.com/yokonad/kody/main/install.sh | bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
GREY='\033[0;90m'
WHITE='\033[1;37m'
RESET='\033[0m'

REPO="yokonad/kody"

# ── Estetica GHOST ──────────────────────────────────────────────────────────
echo ""
printf '%b' "${RED}"
cat <<'GHOST'
  _  __  ___   ____  __   __
 | |/ / / _ \ |  _ \ \ \ / /
 | ' / | | | || | | | \ V /
 | . \ | |_| || |_| |  | |
 |_|\_\ \___/ |____/   |_|
GHOST
printf '%b\n' "${RESET}"
echo -e "${GREY}  private. dangerous. elite.   KODY installer${RESET}"
echo ""

step() { echo -e "${GREY}[ $1 ]${RESET} ... ${GREEN}[OK]${RESET}"; }
step "establishing secure channel"
step "resolving latest release"

# ── Detectar plataforma ─────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"
ASSET=""

case "$OS" in
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            ASSET="kody-x86_64-unknown-linux-musl.tar.gz"
        fi
        ;;
    Darwin)
        if [ "$ARCH" = "arm64" ] || [ "$ARCH" = "aarch64" ]; then
            ASSET="kody-aarch64-apple-darwin.tar.gz"
        fi
        ;;
esac

if [ -z "$ASSET" ]; then
    echo -e "  ${RED}[ERROR] No hay binario pre-compilado para ${OS}/${ARCH}.${RESET}"
    echo -e "  ${CYAN}Como alternativa, compila desde codigo fuente:${RESET}"
    echo -e "    ${WHITE}git clone https://github.com/${REPO}.git${RESET}"
    echo -e "    ${WHITE}cd kody/kody && cargo build --release${RESET}"
    exit 1
fi

URL="https://github.com/${REPO}/releases/latest/download/${ASSET}"
TMPDIR="$(mktemp -d)"
trap 'rm -rf "$TMPDIR"' EXIT

# ── PASO 1: Descargar binario pre-compilado ─────────────────────────────────
echo ""
echo -e "${MAGENTA}[PASO 1] Descargando Kody (binario pre-compilado)...${RESET}"
echo -e "  ${GREY}[debug] origen: ${URL}${RESET}"

HTTP_CODE="$(curl -fsSL -w '%{http_code}' -o "$TMPDIR/kody.tar.gz" "$URL" 2>/dev/null || echo "000")"

if [ ! -s "$TMPDIR/kody.tar.gz" ]; then
    if [ "$HTTP_CODE" = "404" ]; then
        echo -e "  ${RED}[ERROR] No se encontro una version pre-compilada (HTTP 404).${RESET}"
        echo -e "  ${YELLOW}Aun no hay un release publicado para tu plataforma.${RESET}"
    else
        echo -e "  ${RED}[ERROR] Fallo de red al descargar (codigo: ${HTTP_CODE}).${RESET}"
    fi
    echo -e "  ${CYAN}Como alternativa, compila desde codigo fuente:${RESET}"
    echo -e "    ${WHITE}git clone https://github.com/${REPO}.git${RESET}"
    echo -e "    ${WHITE}cd kody/kody && cargo build --release${RESET}"
    exit 1
fi

echo -e "  ${GREEN}[OK] Descarga completada${RESET}"

# ── PASO 2: Extraer e instalar ──────────────────────────────────────────────
echo ""
echo -e "${MAGENTA}[PASO 2] Instalando...${RESET}"

tar -xzf "$TMPDIR/kody.tar.gz" -C "$TMPDIR"

BIN_DIR="$HOME/.local/bin"
BIN_PATH="$BIN_DIR/kody"
mkdir -p "$BIN_DIR"

EXTRACTED="$(find "$TMPDIR" -name kody -type f | head -1)"
if [ -z "$EXTRACTED" ]; then
    echo -e "  ${RED}[ERROR] No se encontro el binario 'kody' en el archivo.${RESET}"
    exit 1
fi

cp "$EXTRACTED" "$BIN_PATH"
chmod +x "$BIN_PATH"
echo -e "  ${GREEN}[OK] Kody instalado en: $BIN_PATH${RESET}"

# ── PASO 3: Configurar PATH ─────────────────────────────────────────────────
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    SHELL_RC="$HOME/.bashrc"
    [ -f "$HOME/.zshrc" ] && SHELL_RC="$HOME/.zshrc"
    if ! grep -q '.local/bin' "$SHELL_RC" 2>/dev/null; then
        {
            echo ''
            echo '# Kody binary'
            echo 'export PATH="$HOME/.local/bin:$PATH"'
        } >> "$SHELL_RC"
    fi
    export PATH="$BIN_DIR:$PATH"
    echo -e "  ${GREEN}[OK] PATH actualizado${RESET}"
fi

echo ""
echo -e "${GREEN}INSTALACION COMPLETADA!${RESET}"
echo ""
echo -e "${CYAN}Abre una NUEVA terminal y ejecuta:${RESET}"
echo -e "  ${WHITE}kody --help${RESET}"
echo -e "  ${WHITE}kody buscar ejemplo.com${RESET}"
echo ""
