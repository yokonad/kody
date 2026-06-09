# Kody — Scanner de Vulnerabilidades CLI

```
  _  __  ___   ____  __   __
 | |/ / / _ \ |  _ \ \ \ / /
 | ' / | | | || | | | \ V /
 | . \ | |_| || |_| |  | |
 |_|\_\ \___/ |____/   |_|
  private. dangerous. elite.
```

Herramienta CLI en Rust para reconocimiento y escaneo de vulnerabilidades, con
detección **real** de servicios (banner grabbing), una base de **CVEs curada y
verificada**, e integración de IA que **autodetecta el proveedor** por la clave.

## Filosofía

- **Simple**: nada de la complejidad de nmap. `kody buscar <objetivo>` y listo.
- **Real**: detecta el producto y versión que de verdad corre en cada puerto y
  solo reporta una CVE cuando esa versión está realmente en el rango vulnerable.
  Sin versión → "exposiciones" honestas, **nunca CVEs inventadas**.
- **Rápido**: por defecto escanea los ~120 puertos más comunes con alta
  concurrencia. `--ports full` para los 65535 cuando lo necesites.

## Instalación con Un Solo Comando

### Linux / macOS

```bash
curl -fsSL https://raw.githubusercontent.com/yokonad/kody/main/install.sh | bash
```

### Windows (PowerShell)

```powershell
irm https://raw.githubusercontent.com/yokonad/kody/main/install.ps1 | iex
```

El instalador:

1. Descarga el binario **pre-compilado** desde GitHub Releases (**~10 segundos**)
2. Lo extrae y lo instala en tu `PATH`
3. **No requiere Rust, ni Git, ni compilación**

---

## Instalación Manual (desde código fuente)

Solo si tu plataforma no tiene binario pre-compilado. Aquí sí necesitas el
toolchain de Rust (desde <https://rustup.rs>):

```bash
git clone https://github.com/yokonad/kody.git
cd kody/kody
cargo build --release
./target/release/kody --help
```

## Uso Rápido

```bash
# Buscar todo sobre un dominio (resuelve TODAS sus IPs + servicios + CVEs)
kody buscar ejemplo.com

# Buscar sobre una IP concreta
kody buscar 192.168.1.1

# Escaneo completo de puertos (1-65535)
kody buscar ejemplo.com --ports full

# Con análisis de IA (te pedirá la API key si no hay una configurada)
kody buscar 192.168.1.1 --ai

# Descubrir y listar TODAS las IPs de tu red local
kody red

# Mapear IPs ocultas / puertos no estándar
kody buscar ocultas --range 192.168.1.0/24
```

## Comandos

### `kody buscar <objetivo>`  *(alias: `scan`)*

Reconocimiento de una IP o un dominio.

- **Dominio**: resuelve y muestra **todas** las IPs a las que apunta, luego
  escanea puertos, detecta servicios+versiones reales y cruza con la base de
  CVEs. "Todo sobre él".
- **IP**: escanea directamente.

```bash
kody buscar ejemplo.com
kody buscar 10.0.0.5 --ports 1-1024
kody buscar api.ejemplo.com --ai
```

| Opción | Descripción |
|--------|-------------|
| `--ports <spec>` | `top` (por defecto), `full` (1-65535), o `80,443,8000-8010` |
| `--ai` | Análisis con IA de los resultados |

### `kody red`  *(alias: `auto-scan`)*

Descubre los dispositivos de tu red local y al final te entrega un inventario
claro de **"las IPs que tengo"**, con puertos y hallazgos por host.

```bash
kody red
kody red --interface eth0
```

### `kody buscar ocultas`

Mapea IPs "ocultas" o con puertos no estándar (cámaras, routers, backdoors).

```bash
kody buscar ocultas --range 192.168.1.0/24
kody buscar ocultas --range 192.168.1.0/24 --deep
```

### `kody config`

Configura la IA y el formato de salida.

```bash
# Pega tu clave: el proveedor se DETECTA solo (OpenAI / Anthropic / Google)
kody config --ai-key TU_CLAVE

# Ver configuración actual
kody config --show
```

## Integración de IA (sin proveedor fijo)

Kody no está atado a una IA concreta. Cuando usas `--ai`:

1. Si ya tienes una clave configurada, se usa.
2. Si no, te **pide pegar una API key** (o Enter para modo offline).
3. El sistema **identifica automáticamente** el proveedor por el formato:
   - `sk-ant-...` → **Anthropic** (Claude)
   - `AIza...` → **Google** (Gemini)
   - `sk-...` → **OpenAI** (GPT)
4. La clave se guarda para la próxima vez.

Sin clave, el **modo offline** genera el informe con la base de CVEs local.

## Detección real de vulnerabilidades

- **Banner grabbing**: lee el banner de cada puerto abierto (SSH/FTP/SMTP por el
  saludo de conexión; HTTP/HTTPS por la cabecera `Server`) para obtener el
  **producto y versión reales**.
- **CVEs curadas por versión**: cada CVE de la base está verificada y solo se
  reporta si la versión detectada cae en su rango vulnerable
  (p. ej. regreSSHion en OpenSSH 8.5p1–9.7p1, Apache 2.4.49 path traversal,
  vsftpd 2.3.4 backdoor, SambaCry, Exim, nginx CVE-2021-23017).
- **Exposiciones**: servicios sensibles (Telnet en claro, Redis/Mongo/MySQL
  expuestos, RDP/VNC, SMB…) se reportan como riesgo honesto, **sin CVE inventada**.
- **Análisis web** (puertos HTTP/HTTPS): cabeceras de seguridad ausentes (HSTS,
  CSP, X-Frame-Options, nosniff), redirección HTTP→HTTPS, cookies inseguras y
  divulgación de versión.
- **Impacto y ubicación**: cada hallazgo explica **qué podría dañar un atacante**
  y **dónde** (puerto, cabecera o servicio).
- **Superficie de ataque**: resumen final con riesgo global, puntos de entrada y
  una lista priorizada de **qué arreglar primero**.

## Arquitectura

```
kody/
├── src/
│   ├── main.rs          # CLI + estética GHOST (banner, boot, tabla de sesión)
│   ├── ascii/           # Banner, secuencia de arranque, tabla de sesión
│   ├── ai/              # Proveedores de IA (autodetección) + modo offline
│   ├── scanner/         # Escaneo de puertos, banner grabbing, base de CVEs
│   ├── network/         # Descubrimiento de red (detección de IP local real)
│   ├── db/              # Cache offline con SQLite
│   └── config/          # Configuración + autodetección de proveedor
├── install.sh           # Instalador Linux/macOS (descarga binario)
└── install.ps1          # Instalador Windows (descarga binario)
```

## Nota de Seguridad

La clave de API se guarda en texto plano en `~/.kody/config.toml`. En la base de
datos (`~/.kody/methods.db`) solo se guardan **hashes** de las claves, nunca la
clave en claro. Usa Kody solo contra objetivos para los que tengas autorización.

## Licencia

MIT License — ver archivo LICENSE.

## Autores

Kody Team
