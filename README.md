# Kody - Vulnerability Scanner CLI

A Rust-powered CLI tool for vulnerability scanning with AI integration.

## Features

- **IP/Domain Scanning** - Scan specific targets for vulnerabilities
- **Auto-Scan** - Automatically discover and scan all devices on your network
- **Hidden IP Mapping** - Discover IPs with non-standard port configurations
- **AI Integration** - Optional AI analysis using OpenAI or Anthropic
- **Offline Mode** - Works without internet using cached vulnerability patterns
- **Cross-Platform** - Works on Linux, macOS, and Windows

## Requirements

- Rust 1.70+ ([install rust](https://rustup.rs))
- SQLite (usually pre-installed on Linux/macOS)
- Network access for scanning

## Installation

### Linux/macOS (From Source)

```bash
# Clone the repository
git clone https://github.com/kody-team/kody.git
cd kody/kody

# Build release version
cargo build --release

# Run
./target/release/kody --help
```

### Windows (From Source)

```powershell
# Clone the repository
git clone https://github.com/kody-team/kody.git
cd kody/kody

# Build release version (requires Rust)
cargo build --release

# Run
.\target\release\kody.exe --help
```

### Quick Start

```bash
# See all commands
./target/release/kody --help

# Scan a target
./target/release/kody scan 192.168.1.1 --ports 1-1024

# Auto-discover and scan your network
./target/release/kody auto-scan

# Map hidden IPs in your network
./target/release/kody map-hidden --range 192.168.1.0/24
```

## Usage

### Scan a specific target

```bash
kody scan 192.168.1.1 --ports 1-1024
kody scan example.com --ports 80,443,8080
```

### Auto-scan your network

```bash
kody auto-scan
kody auto-scan --interface eth0
```

### Map hidden IPs (non-standard ports)

```bash
kody map-hidden --range 192.168.1.0/24
kody map-hidden --range 192.168.1.0/24 --deep
```

### Configure AI integration

```bash
kody config --ai-provider openai --ai-key sk-your-key
kody config --show
```

## Commands

| Command | Description |
|---------|-------------|
| `kody scan <target>` | Scan IP or domain for vulnerabilities |
| `kody auto-scan` | Discover and scan all devices on local network |
| `kody map-hidden <range>` | Map hidden/subterranean IPs in CIDR range |
| `kody config` | Configure AI provider and API key |

## Options

- `--ports <range>` - Port range to scan (default: 1-1024)
- `--ai` - Enable AI analysis for scan results
- `--deep` - Deep scan mode for map-hidden
- `--json` - Output results in JSON format

## Architecture

```
kody/
├── src/
│   ├── main.rs         # CLI entry point
│   ├── ascii/          # ASCII art banners
│   ├── ai/             # AI providers (OpenAI, Anthropic, offline)
│   ├── scanner/        # Port scanning & vulnerability detection
│   ├── network/        # Network discovery
│   ├── db/             # SQLite offline cache
│   └── config/         # Configuration management
└── Cargo.toml          # Rust dependencies
```

## Security Note

Tokens are stored in plaintext in `~/.kody/methods.db`. Future versions will include encryption at rest.

## License

MIT License - see LICENSE file

## Authors

Kody Team