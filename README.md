# Kody - Vulnerability Scanner CLI

A Rust-powered CLI tool for vulnerability scanning with AI integration.

## Features

- **IP/Domain Scanning** - Scan specific targets for vulnerabilities
- **Auto-Scan** - Automatically discover and scan all devices on your network
- **Hidden IP Mapping** - Discover IPs with non-standard port configurations
- **AI Integration** - Optional AI analysis using OpenAI or Anthropic
- **Offline Mode** - Works without internet using cached vulnerability patterns
- **Cross-Platform** - Works on Linux, macOS, and Windows (PowerShell)

## Installation

### Linux/macOS

```bash
curl -fsSL https://kody.dev/install | sh
```

### Windows (PowerShell)

```powershell
irm https://kody.dev/install | iex
```

### From Source

```bash
git clone https://github.com/kody-team/kody.git
cd kody
cargo build --release
./target/release/kody --help
```

## Usage

### Scan a target

```bash
kody scan 192.168.1.1 --ports 1-1024
```

### Auto-scan your network

```bash
kody auto-scan
```

### Map hidden IPs

```bash
kody map-hidden --range 192.168.1.0/24
kody map-hidden --range 192.168.1.0/24 --deep  # deeper scan
```

### Configure AI integration

```bash
kody config --ai-provider openai --ai-key sk-your-key
kody config --show
```

## Commands

- `kody scan <target>` - Scan IP or domain for vulnerabilities
- `kody auto-scan` - Discover and scan all devices on local network
- `kody map-hidden <range>` - Map hidden/subterranean IPs in CIDR range
- `kody config` - Configure AI provider and API key

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
│   ├── ai/             # AI providers (OpenAI, offline)
│   ├── scanner/        # Port scanning & vulnerability detection
│   ├── network/         # Network discovery
│   ├── db/             # SQLite offline cache
│   └── config/          # Configuration management
└── install.sh          # Bootstrap installer
```

## Security Note

Tokens are stored in plaintext in `~/.kody/methods.db`. Future versions will include encryption at rest.

## License

MIT License - see LICENSE file

## Authors

Kody Team