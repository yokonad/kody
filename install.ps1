#!/usr/bin/env pwsh
# Kody Installer - PowerShell bootstrap script for Windows
# Usage: iwr https://kody.dev/install | iex
# Or:   irm https://kody.dev/install | iex

$ErrorActionPreference = "Stop"

$KODY_VERSION = "0.1.0"
$INSTALL_DIR = "$env:USERPROFILE\.kody\bin"
$RELEASES_URL = "https://github.com/kody-team/kody/releases/latest/download"

function Detect-OS {
    if ($env:OS -eq "Windows_NT") {
        return "windows"
    }
    return "unsupported"
}

function Detect-Arch {
    switch ($env:PROCESSOR_ARCHITECTURE) {
        "AMD64" { return "amd64" }
        "ARM64" { return "arm64" }
        "X86"   { return "386" }
        default { return "amd64" }
    }
}

function Install-Kody {
    $os = Detect-OS
    $arch = Detect-Arch

    if ($os -eq "unsupported") {
        Write-Error "Unsupported operating system"
        exit 1
    }

    Write-Host "[*] Detected OS: $os ($arch)" -ForegroundColor Cyan

    # Create install directory
    if (-not (Test-Path $INSTALL_DIR)) {
        New-Item -ItemType Directory -Path $INSTALL_DIR -Force | Out-Null
    }

    $binary_name = "kody-${os}-${arch}.exe"
    $download_url = "${RELEASES_URL}/${binary_name}"
    $target_path = Join-Path $INSTALL_DIR "kody.exe"

    Write-Host "[*] Downloading from: $download_url" -ForegroundColor Cyan
    Write-Host "[*] Installing to: $target_path" -ForegroundColor Cyan

    try {
        Invoke-WebRequest -Uri $download_url -OutFile $target_path -UseBasicParsing
    } catch {
        Write-Error "Failed to download: $_"
        exit 1
    }

    # Add to PATH for current session
    $currentPath = $env:PATH
    if ($currentPath -notlike "*$INSTALL_DIR*") {
        $env:PATH = "$INSTALL_DIR;$currentPath"
        Write-Host "[*] Added $INSTALL_DIR to PATH for current session" -ForegroundColor Yellow
    }

    # Persist to user PATH permanently
    $userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
    if ($userPath -notlike "*$INSTALL_DIR*") {
        [Environment]::SetEnvironmentVariable("PATH", "$INSTALL_DIR;$userPath", "User")
        Write-Host "[*] Added $INSTALL_DIR to user PATH permanently" -ForegroundColor Yellow
    }

    Write-Host ""
    Write-Host "[+] Installation complete!" -ForegroundColor Green
    Write-Host "[*] Run 'kody --help' to get started" -ForegroundColor Cyan
    Write-Host ""
}

function Check-Version {
    Write-Host "Kody v${KODY_VERSION}"
}

$args = $args[0]

switch ($args) {
    { $_ -eq "--version" -or $_ -eq "-v" } { Check-Version }
    { $_ -eq "--help" -or $_ -eq "-h" } {
        Write-Host "Kody Installer"
        Write-Host "Usage: irm https://kody.dev/install | iex"
        Write-Host ""
        Write-Host "Options:"
        Write-Host "  --version, -v  Show version"
        Write-Host "  --help, -h     Show this help"
    }
    default {
        Write-Host "Installing Kody v${KODY_VERSION}..." -ForegroundColor Cyan
        Install-Kody
    }
}