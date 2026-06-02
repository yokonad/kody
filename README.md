# Kody - Scanner de Vulnerabilidades CLI

Herramienta CLI desarrollada en Rust para escaneo de vulnerabilidades con integración de IA.

## Características

- **Escaneo de IP/Dominio** - Escanea objetivos específicos en busca de vulnerabilidades
- **Auto-Escaneo** - Descubre y escanea automáticamente todos los dispositivos en tu red
- **Mapeo de IPs Ocultas** - Descubre IPs con configuraciones de puertos no estándar
- **Integración con IA** - Análisis opcional mediante IA usando OpenAI o Anthropic
- **Modo Sin Conexión** - Funciona sin internet usando patrones de vulnerabilidades cacheados
- **Multiplataforma** - Funciona en Linux, macOS y Windows

## Requisitos

- Rust 1.70+ ([instalar Rust](https://rustup.rs))
- SQLite (generalmente preinstalado en Linux/macOS)
- Acceso a la red para realizar escaneos

## Instalación

### Linux / macOS (desde código fuente)

```bash
# Clonar el repositorio
git clone https://github.com/yokonad/kody.git
cd kody/kody

# Compilar versión de producción
cargo build --release

# Ejecutar
./target/release/kody --help
```

### Windows (desde código fuente)

```powershell
# Clonar el repositorio
git clone https://github.com/yokonad/kody.git
cd kody/kody

# Compilar versión de producción (requiere Rust)
cargo build --release

# Ejecutar
.\target\release\kody.exe --help
```

## Uso Rápido

```bash
# Ver todos los comandos disponibles
./target/release/kody --help

# Escanear un objetivo específico
./target/release/kody scan 192.168.1.1 --ports 1-1024

# Auto-descubrir y escanear tu red
./target/release/kody auto-scan

# Mapear IPs ocultas en tu red
./target/release/kody map-hidden --range 192.168.1.0/24
```

## Comandos

### `kody scan <objetivo>`

Escanea una IP o dominio específico en busca de vulnerabilidades.

```bash
# Escanear puertos comunes
kody scan 192.168.1.1 --ports 1-1024

# Escanear puertos específicos
kody scan example.com --ports 80,443,8080

# Escanear con análisis de IA
kody scan 192.168.1.1 --ports 1-1024 --ai
```

### `kody auto-scan`

Descubre automáticamente todos los dispositivos en tu red local y los escanea.

```bash
# Escanear red automáticamente
kody auto-scan

# Escanear usando una interfaz específica
kody auto-scan --interface eth0
```

### `kody map-hidden <rango>`

Mapea IPs "ocultas" o con puertos no estándar (común en sistemas de vigilancia, cámaras, etc.).

```bash
# Mapeo básico
kody map-hidden --range 192.168.1.0/24

# Mapeo profundo (más lento pero más exhaustivo)
kody map-hidden --range 192.168.1.0/24 --deep
```

### `kody config`

Configura el proveedor de IA y la clave API.

```bash
# Configurar OpenAI
kody config --ai-provider openai --ai-key sk-tu-clave

# Ver configuración actual
kody config --show
```

## Opciones

| Opción | Descripción |
|--------|-------------|
| `--ports <rango>` | Rango de puertos a escanear (por defecto: 1-1024) |
| `--ai` | Habilitar análisis con IA para los resultados |
| `--deep` | Modo de escaneo profundo para map-hidden |
| `--json` | Salida de resultados en formato JSON |
| `--interface <nombre>` | Interfaz de red a usar para auto-scan |

## Arquitectura

```
kody/
├── src/
│   ├── main.rs         # Punto de entrada CLI
│   ├── ascii/          # Arte ASCII para banners
│   ├── ai/             # Proveedores de IA (OpenAI, Anthropic, offline)
│   ├── scanner/        # Escaneo de puertos y detección de vulnerabilidades
│   ├── network/        # Descubrimiento de red
│   ├── db/             # Cache offline con SQLite
│   └── config/         # Gestión de configuración
└── Cargo.toml          # Dependencias Rust
```

## Nota de Seguridad

Los tokens se almacenan en texto plano en `~/.kody/methods.db`. Las versiones futuras incluirán cifrado en reposo.

## Licencia

MIT License - ver archivo LICENSE

## Autores

Kody Team