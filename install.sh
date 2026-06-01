#!/usr/bin/env bash
# Kody Installer - Bootstrap script for cross-platform CLI installation
# Usage: curl -fsSL https://kody.dev/install | sh
# Windows: iwr https://kody.dev/install | iex

set -e

KODY_VERSION="0.1.0"
INSTALL_DIR="${HOME}/.kody/bin"
RELEASES_URL="https://github.com/kody-team/kody/releases/latest/download"

detect_os() {
    case "$(uname -s)" in
        Linux*)     echo "linux" ;;
        Darwin*)    echo "darwin" ;;
        CYGWIN*|MSYS*|MINGW*) echo "windows" ;;
        *)          echo "unsupported" ;;
    esac
}

detect_arch() {
    case "$(uname -m)" in
        x86_64)     echo "amd64" ;;
        aarch64|arm64) echo "arm64" ;;
        i386|i686)  echo "386" ;;
        *)          echo "amd64" ;;
    esac
}

install_kody() {
    local os=$(detect_os)
    local arch=$(detect_arch)

    if [ "$os" == "unsupported" ]; then
        echo "Error: Unsupported operating system"
        exit 1
    fi

    echo "[*] Detected OS: $os ($arch)"

    # Create install directory
    mkdir -p "$INSTALL_DIR"

    local binary_name="kody-${os}-${arch}"
    local extension=""
    if [ "$os" == "windows" ]; then
        extension=".exe"
        binary_name="${binary_name}.exe"
    fi

    local download_url="${RELEASES_URL}/${binary_name}"
    local target_path="${INSTALL_DIR}/kody${extension}"

    echo "[*] Downloading from: $download_url"
    echo "[*] Installing to: $target_path"

    # Download binary
    if command -v curl > /dev/null 2>&1; then
        curl -fsSL "$download_url" -o "$target_path"
    elif command -v wget > /dev/null 2>&1; then
        wget -q "$download_url" -O "$target_path"
    else
        echo "Error: curl or wget is required"
        exit 1
    fi

    # Make executable
    chmod +x "$target_path"

    # Add to PATH if not already there
    local path_entry="export PATH=\"\$PATH:${INSTALL_DIR}\""
    local shell_rc=""

    case "$(basename "${SHELL:-bash}")" in
        bash) shell_rc="${HOME}/.bashrc" ;;
        zsh) shell_rc="${HOME}/.zshrc" ;;
        fish) shell_rc="${HOME}/.config/fish/config.fish" ;;
        *) shell_rc="${HOME}/.profile" ;;
    esac

    if ! grep -q "$INSTALL_DIR" "$shell_rc" 2>/dev/null; then
        echo "[*] Adding ${INSTALL_DIR} to PATH in $shell_rc"
        echo "$path_entry" >> "$shell_rc"
        echo "[*] Run 'source $shell_rc' or start a new shell to use kody"
    fi

    echo ""
    echo "[+] Installation complete!"
    echo "[*] Run 'kody --help' to get started"
    echo ""
}

# Check for updates
check_version() {
    echo "Kody v${KODY_VERSION}"
}

# Main
case "${1:-}" in
    --version|-v) check_version ;;
    --help|-h)
        echo "Kody Installer"
        echo "Usage: curl -fsSL https://kody.dev/install | sh"
        echo ""
        echo "Options:"
        echo "  --version, -v  Show version"
        echo "  --help, -h     Show this help"
        ;;
    *)
        echo "Installing Kody v${KODY_VERSION}..."
        install_kody
        ;;
esac