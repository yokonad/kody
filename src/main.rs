use clap::{Parser, Subcommand};
use sha2::{Digest, Sha256};
use std::io::Write;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{info, warn};

mod ascii;
mod cli;
mod config;
mod ai;
mod scanner;
mod db;
mod network;

pub use ascii::banner;
pub use config::Settings;

// ANSI color codes
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const GREY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

#[derive(Parser)]
#[command(
    name = "kody",
    author = "Kody Team",
    version,
    about = "Ghost-grade vulnerability scanner with auto-detecting AI",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Buscar: recon de un objetivo (IP/dominio), o un sub-modo:
    /// `kody buscar ocultas` / `kody buscar red`
    #[command(visible_alias = "scan")]
    Buscar {
        /// IP, dominio, o un sub-modo: "ocultas" (IPs ocultas) o "red" (tu red)
        target: String,
        /// [objetivo] Puertos: "top" (por defecto), "full" (1-65535), o "80,443"
        #[arg(short, long, default_value = "top")]
        ports: String,
        /// Habilitar análisis con IA
        #[arg(short, long)]
        ai: bool,
        /// [ocultas] Rango de red en notación CIDR
        #[arg(long, default_value = "192.168.0.0/24")]
        range: String,
        /// [ocultas] Escaneo profundo (puertos subterráneos)
        #[arg(short, long)]
        deep: bool,
        /// [red] Interfaz de red a usar (auto-detecta si se omite)
        #[arg(short, long)]
        interface: Option<String>,
    },
    /// Descubrir y escanear todos los dispositivos de tu red local
    #[command(visible_alias = "auto-scan")]
    Red {
        /// Interfaz de red a usar (auto-detecta si se omite)
        #[arg(short, long)]
        interface: Option<String>,
        /// Habilitar análisis con IA
        #[arg(short, long)]
        ai: bool,
    },
    /// Configurar IA (la clave se autodetecta) y formato de salida
    Config {
        /// Forzar proveedor (openai, anthropic, google) — normalmente innecesario
        #[arg(long)]
        ai_provider: Option<String>,
        /// Clave de API de IA (el proveedor se detecta automáticamente)
        #[arg(long)]
        ai_key: Option<String>,
        /// Formato de salida (text, json)
        #[arg(long)]
        output: Option<String>,
        /// Mostrar configuración actual
        #[arg(short, long)]
        show: bool,
    },
}

/// Get database path in ~/.kody/methods.db
fn get_db_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".kody").join("methods.db")
}

/// Initialize database with graceful fail
fn init_db() -> Option<db::Database> {
    let db_path = get_db_path();
    if let Some(parent) = db_path.parent() {
        if !parent.exists() && std::fs::create_dir_all(parent).is_err() {
            warn!("Could not create database directory: {:?}", parent);
            return None;
        }
    }
    match db::Database::new(&db_path) {
        Ok(database) => {
            info!("Database initialized at {:?}", db_path);
            Some(database)
        }
        Err(e) => {
            warn!("Could not initialize database: {}. Continuing without DB.", e);
            None
        }
    }
}

/// Derive a stable, anonymous operator handle for the session banner.
fn operator_handle() -> String {
    let seed = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "anon".to_string());
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    let hex = hex::encode(hasher.finalize());
    format!("GHOST-{}", hex[..4].to_uppercase())
}

/// Detect the local egress IP and mask the last two octets for the banner.
fn masked_egress() -> String {
    match network::detect_local_ipv4() {
        Some(ip) => {
            let o = ip.octets();
            format!("{}.{}.***.***", o[0], o[1])
        }
        None => "[OFFLINE]".to_string(),
    }
}

fn main() -> ExitCode {
    // GHOST-style startup.
    println!("{}", banner());
    ascii::boot_sequence();

    let db = init_db();

    ascii::session_table(&operator_handle(), &masked_egress(), scanner::ttp_count());

    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    let result = match cli.command {
        Commands::Buscar { target, ports, ai, range, deep, interface } => {
            // `buscar` is the umbrella verb: a bare target does recon, while the
            // keywords "ocultas" and "red" switch to those modes.
            match target.as_str() {
                "ocultas" => rt.block_on(async_map_hidden(&range, deep)),
                "red" => rt.block_on(async_auto_scan(interface.as_deref(), ai)),
                _ => rt.block_on(async_scan(&target, &ports, ai, db.as_ref())),
            }
        }
        Commands::Red { interface, ai } => rt.block_on(async_auto_scan(interface.as_deref(), ai)),
        Commands::Config { ai_provider, ai_key, output, show } => handle_config(ai_provider, ai_key, output, show, db.as_ref()),
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}Error:{} {}", RED, e, RESET);
            ExitCode::FAILURE
        }
    }
}

fn print_vuln_line(vuln: &scanner::Vulnerability) {
    let color = match vuln.severity {
        scanner::Severity::Critical => RED,
        scanner::Severity::High => "\x1b[91m",
        scanner::Severity::Medium => YELLOW,
        scanner::Severity::Low => "\x1b[34m",
        scanner::Severity::Info => RESET,
    };
    print!("{}│    [{}{}{}]", CYAN, color, vuln.severity, RESET);
    if let Some(cve) = &vuln.cve_id {
        print!(" {}{}{}", color, cve, RESET);
    }
    println!(" {}(port {}){}", GREY, vuln.affected_port, RESET);
    println!("{}│      {}{}", CYAN, vuln.description, RESET);
}

async fn async_scan(target: &str, ports: &str, ai_analysis: bool, db: Option<&db::Database>) -> anyhow::Result<()> {
    let is_domain = !scanner::is_ip_literal(target);

    if is_domain {
        println!("{}~[~] Investigando dominio: {}{}", YELLOW, target, RESET);
    } else {
        println!("{}~[~] Investigando IP: {}{}", YELLOW, target, RESET);
    }

    // For a domain, resolve and report EVERY IP it points to.
    let mut resolved_ips: Vec<String> = Vec::new();
    if is_domain {
        resolved_ips = scanner::resolve_all(target).await;
        if resolved_ips.is_empty() {
            anyhow::bail!("No se pudo resolver el dominio '{}'", target);
        }
        println!("{}*[*] IPs resueltas ({}):{}", CYAN, resolved_ips.len(), RESET);
        for ip in &resolved_ips {
            println!("{}│    -> {}{}", CYAN, ip, RESET);
        }
    }

    let port_list = scanner::parse_port_spec(ports);
    println!("{}*[*] Escaneando {} puertos...{}", CYAN, port_list.len(), RESET);

    let config = scanner::ScanConfig {
        target: target.to_string(),
        ports: ports.to_string(),
        timeout_ms: 1500,
        concurrent: 200,
    };

    let result = scanner::Scanner::run(target, port_list, &config).await;

    // Cache in DB if available.
    if let Some(database) = db {
        let vuln_cache = db::VulnCache::new(database.connection());
        for vuln in &result.vulnerabilities {
            let _ = vuln_cache.cache_vulnerability(
                vuln.cve_id.as_deref(),
                Some(&vuln.description),
                Some(&format!("{}", vuln.severity)),
                Some(&vuln.affected_port.to_string()),
            );
        }
        let history = db::ScanHistory::new(database.connection());
        let _ = history.record_scan(target, "buscar", Some(ports), result.vulnerabilities.len() as i64, ai_analysis, None);
    }

    // Results.
    println!("\n{}┌─ Resultados: {}{}", CYAN, target, RESET);
    println!("{}│{}", CYAN, RESET);
    if is_domain && !resolved_ips.is_empty() {
        println!("{}│  Dominio resuelto a: {:?}{}", CYAN, resolved_ips, RESET);
    }
    println!("{}│  Puertos abiertos: {:?}{}", CYAN, result.open_ports, RESET);

    if !result.services.is_empty() {
        println!("{}│  Servicios:{}", CYAN, RESET);
        for svc in &result.services {
            println!("{}│    - Puerto {}: {}{}", CYAN, svc.port, svc.service, RESET);
        }
    }

    if result.vulnerabilities.is_empty() {
        println!("{}│  Sin hallazgos conocidos.{}", CYAN, RESET);
    } else {
        let cves = result.vulnerabilities.iter().filter(|v| v.cve_id.is_some()).count();
        let exposures = result.vulnerabilities.len() - cves;
        println!("{}│  Hallazgos: {} CVE confirmadas, {} exposiciones{}", CYAN, cves, exposures, RESET);
        for vuln in &result.vulnerabilities {
            print_vuln_line(vuln);
        }
    }

    println!("{}│{}", CYAN, RESET);
    println!("{}└{}[Listo]{}", CYAN, "─".repeat(40), RESET);

    if ai_analysis {
        run_ai_analysis(result).await;
    }

    Ok(())
}

async fn run_ai_analysis(result: ai::ScanResult) {
    println!("\n{}┌─ Análisis de IA{}", CYAN, RESET);
    println!("{}│{}", CYAN, RESET);

    let mut settings = Settings::load().unwrap_or_default();

    // No key configured? Offer to paste one — the provider is auto-detected.
    let mut newly_entered = false;
    if settings.ai_key.is_none() {
        if let Some(key) = prompt_for_api_key() {
            match config::detect_provider(&key) {
                Some(p) => println!("{}│  Proveedor detectado: {}{}", CYAN, p, RESET),
                None => println!("{}│  {}No reconocí el proveedor; usaré formato OpenAI.{}", CYAN, YELLOW, RESET),
            }
            settings.ai_provider = config::detect_provider(&key).map(|s| s.to_string());
            settings.ai_key = Some(key);
            newly_entered = true;
        }
    }

    let provider = ai::create_provider(settings.ai_key.clone(), settings.ai_provider.clone());
    match provider.analyze(result).await {
        Ok(report) => {
            for line in report.lines() {
                println!("{}│  {}{}", CYAN, line, RESET);
            }
            // Persist a freshly-entered, working key for next time.
            if newly_entered && settings.save().is_ok() {
                println!("{}│  {}Clave guardada en ~/.kody/config.toml{}", CYAN, GREEN, RESET);
            }
        }
        Err(e) => {
            println!("{}│  {}Aviso: el análisis de IA falló: {}{}", CYAN, YELLOW, e, RESET);
        }
    }
    println!("{}│{}", CYAN, RESET);
    println!("{}└{}[Listo]{}", CYAN, "─".repeat(40), RESET);
}

/// Prompt for an API key on stdin. Empty input means "offline mode".
fn prompt_for_api_key() -> Option<String> {
    print!("{}│  Pega tu API key de IA (Enter = modo offline): {}", CYAN, RESET);
    let _ = std::io::stdout().flush();
    let mut line = String::new();
    if std::io::stdin().read_line(&mut line).is_err() {
        return None;
    }
    let key = line.trim().to_string();
    if key.is_empty() {
        None
    } else {
        Some(key)
    }
}

async fn async_auto_scan(interface: Option<&str>, _ai_analysis: bool) -> anyhow::Result<()> {
    println!("{}", ascii::auto_scan_banner());

    if let Some(iface) = interface {
        println!("{}~[~] Usando interfaz: {}{}", YELLOW, iface, RESET);
    } else {
        println!("{}*[*] Auto-detectando interfaz de red...{}", CYAN, RESET);
    }

    let config = scanner::ScanConfig {
        target: "auto".to_string(),
        ports: "top".to_string(),
        timeout_ms: 1500,
        concurrent: 200,
    };

    let results = scanner::AutoScanner::scan_network(interface.map(|s| s.to_string()), &config).await;

    // IP inventory — "las IPs que tengo".
    println!("\n{}┌─ IPs que tengo en la red{}", CYAN, RESET);
    println!("{}│{}", CYAN, RESET);
    println!("{}│  Tu IP (egress): {}{}", CYAN, masked_egress(), RESET);
    println!("{}│  Hosts vivos: {}{}", CYAN, results.len(), RESET);
    println!("{}│{}", CYAN, RESET);

    let total_vulns: usize = results.iter().map(|r| r.vulnerabilities.len()).sum();
    for result in &results {
        let open = result.open_ports.len();
        let vulns = result.vulnerabilities.len();
        println!("{}│  • {}{}{}  {}({} puertos, {} hallazgos){}", CYAN, GREEN, result.target, RESET, GREY, open, vulns, RESET);
        for vuln in &result.vulnerabilities {
            print_vuln_line(vuln);
        }
    }

    println!("{}│{}", CYAN, RESET);
    println!("{}│  Total de hallazgos: {}{}", CYAN, total_vulns, RESET);
    println!("{}└{}[Listo]{}", CYAN, "─".repeat(40), RESET);

    Ok(())
}

async fn async_map_hidden(range: &str, deep: bool) -> anyhow::Result<()> {
    println!("{}", ascii::map_hidden_banner());

    println!("{}~[~] Mapeando IPs ocultas en: {}{}", YELLOW, range, RESET);
    if deep {
        println!("{}*[*] Modo profundo activado{}", CYAN, RESET);
    }

    let config = scanner::ScanConfig {
        target: range.to_string(),
        ports: if deep { "full".to_string() } else { "top".to_string() },
        timeout_ms: 1500,
        concurrent: 200,
    };

    let results = scanner::HiddenMapper::map_hidden(range, deep, &config).await;

    println!("\n{}┌─ IPs ocultas descubiertas{}", CYAN, RESET);
    println!("{}│{}", CYAN, RESET);
    println!("{}│  Encontradas: {}{}", CYAN, results.len(), RESET);

    for result in &results {
        println!("{}│{}", CYAN, RESET);
        println!("{}│  {} - {} puertos{}", CYAN, result.target, result.open_ports.len(), RESET);
        for vuln in &result.vulnerabilities {
            print_vuln_line(vuln);
        }
    }

    println!("{}│{}", CYAN, RESET);
    println!("{}└{}[Listo]{}", CYAN, "─".repeat(40), RESET);

    Ok(())
}

fn handle_config(
    ai_provider: Option<String>,
    ai_key: Option<String>,
    output: Option<String>,
    show: bool,
    db: Option<&db::Database>,
) -> anyhow::Result<()> {
    let mut settings = Settings::load()?;

    if show {
        println!("\n{}┌─ Configuración de Kody{}", CYAN, RESET);
        println!("{}│{}", CYAN, RESET);
        println!("{}│  Proveedor IA: {}{}", CYAN, settings.ai_provider.as_deref().unwrap_or("(autodetectar)"), RESET);
        println!("{}│  Clave IA: {}{}", CYAN, if settings.ai_key.is_some() { "****" } else { "sin configurar" }, RESET);
        println!("{}│  Salida: {}{}", CYAN, settings.output_format.as_deref().unwrap_or("text"), RESET);

        if let Some(database) = db {
            let path = get_db_path();
            println!("{}│  Base de datos: {}{}", CYAN, path.display(), RESET);
            let vuln_cache = db::VulnCache::new(database.connection());
            if let Ok(count) = vuln_cache.count() {
                println!("{}│    - Vulnerabilidades cacheadas: {}{}", CYAN, count, RESET);
            }
        } else {
            println!("{}│  Base de datos: no inicializada (modo offline){}", CYAN, RESET);
        }

        println!("{}│{}", CYAN, RESET);
        println!("{}└{}[Listo]{}", CYAN, "─".repeat(30), RESET);
        return Ok(());
    }

    let mut changed = false;

    if let Some(key) = ai_key {
        // Auto-detect the provider unless one was explicitly forced.
        let detected = config::detect_provider(&key);
        match (&ai_provider, detected) {
            (None, Some(p)) => {
                settings.ai_provider = Some(p.to_string());
                println!("{}+[+] Proveedor detectado automáticamente: {}{}", GREEN, p, RESET);
            }
            (None, None) => {
                println!("{}~[~] No reconocí el formato de la clave; se usará formato OpenAI por defecto.{}", YELLOW, RESET);
            }
            _ => {}
        }
        settings.ai_key = Some(key);
        println!("{}[+] Clave de API guardada{}", GREEN, RESET);
        changed = true;
    }

    if let Some(provider) = ai_provider {
        settings.ai_provider = Some(provider.clone());
        println!("{}+[+] Proveedor IA forzado a: {}{}", GREEN, provider, RESET);
        changed = true;
    }

    if let Some(fmt) = output {
        settings.output_format = Some(fmt.clone());
        println!("{}[+] Formato de salida: {}{}", GREEN, fmt, RESET);
        changed = true;
    }

    if changed {
        settings.save()?;
        println!("{}*[*] Configuración guardada en ~/.kody/config.toml{}", CYAN, RESET);
    }

    Ok(())
}
