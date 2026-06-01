use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{info, warn};

mod ascii;
mod cli;
mod config;
mod ai;
mod scanner;
mod db;

pub use ascii::banner;
pub use config::Settings;

// ANSI color codes
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const CYAN: &str = "\x1b[36m";
const RESET: &str = "\x1b[0m";

#[derive(Parser)]
#[command(
    name = "kody",
    author = "Kody Team",
    version,
    about = "Vulnerability scanner CLI with AI integration",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Scan a specific IP or domain for vulnerabilities
    Scan {
        /// Target IP address or domain name
        target: String,
        /// Port range to scan (e.g., 1-1024)
        #[arg(short, long, default_value = "1-1024")]
        ports: String,
        /// Enable AI analysis
        #[arg(short, long)]
        ai: bool,
    },
    /// Automatically discover and scan all devices on the local network
    AutoScan {
        /// Network interface to use (auto-detect if not specified)
        #[arg(short, long)]
        interface: Option<String>,
        /// Enable AI analysis
        #[arg(short, long)]
        ai: bool,
    },
    /// Map hidden and subterranean IPs on the network
    MapHidden {
        /// Network range to scan (CIDR notation)
        #[arg(short, long, default_value = "192.168.0.0/24")]
        range: String,
        /// Scan for deep/subterranean ports
        #[arg(short, long)]
        deep: bool,
    },
    /// Configure AI provider and API key
    Config {
        /// Set the AI provider (openai, anthropic)
        #[arg(long)]
        ai_provider: Option<String>,
        /// Set the AI API key
        #[arg(long)]
        ai_key: Option<String>,
        /// Set output format (text, json)
        #[arg(long)]
        output: Option<String>,
        /// Show current configuration
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

    // Ensure directory exists
    if let Some(parent) = db_path.parent() {
        if !parent.exists() {
            if std::fs::create_dir_all(parent).is_err() {
                warn!("Could not create database directory: {:?}", parent);
                return None;
            }
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

fn main() -> ExitCode {
    // Show ASCII banner
    println!("{}", banner());

    // Initialize database (graceful fail - Option)
    let db = init_db();

    let cli = Cli::parse();
    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");

    let result = match cli.command {
        Commands::Scan { target, ports, ai } => rt.block_on(async_scan(&target, &ports, ai, db.as_ref())),
        Commands::AutoScan { interface, ai } => rt.block_on(async_auto_scan(interface.as_deref(), ai)),
        Commands::MapHidden { range, deep } => rt.block_on(async_map_hidden(&range, deep)),
        Commands::Config { ai_provider, ai_key, output, show } => handle_config(ai_provider, ai_key, output, show, db.as_ref()),
    };

    match result {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("{}{}Error:{} {}", RED, RESET, e);
            ExitCode::FAILURE
        }
    }
}

async fn async_scan(target: &str, ports: &str, ai_analysis: bool, db: Option<&db::Database>) -> anyhow::Result<()> {
    println!("{}~[~] Scanning target: {} on ports {}{}", YELLOW, target, ports, RESET);
    if ai_analysis {
        println!("{}*[*] AI analysis enabled{}", CYAN, RESET);
    }

    let config = scanner::ScanConfig {
        target: target.to_string(),
        ports: ports.to_string(),
        timeout_ms: 3000,
        concurrent: 100,
    };

    let port_list = scanner::parse_port_range(ports);
    println!("{}*[*] Scanning {} ports...{}", CYAN, port_list.len(), RESET);

    let result = scanner::Scanner::run(target, port_list, &config).await;

    // Cache vulnerabilities in DB if available
    if let Some(database) = db {
        if let Ok(vuln_cache) = Ok(db::VulnCache::new(database.connection())) {
            for vuln in &result.vulnerabilities {
                let _ = vuln_cache.cache_vulnerability(
                    vuln.cve_id.as_deref(),
                    Some(&vuln.description),
                    Some(&format!("{}", vuln.severity)),
                    Some(&vuln.affected_port.to_string()),
                );
            }
        }
        // Record scan in history
        if let Ok(history) = Ok(db::ScanHistory::new(database.connection())) {
            let _ = history.record_scan(
                target,
                "scan",
                Some(ports),
                result.vulnerabilities.len() as i64,
                ai_analysis,
                None,
            );
        }
    }

    // Print results
    println!("\n{}┌─ Scan Results: {}{}", CYAN, target, RESET);
    println!("{}│{}", CYAN, RESET);
    println!("{}│  Open ports: {:?}{}", CYAN, result.open_ports, RESET);

    if !result.services.is_empty() {
        println!("{}│  Services:{}{}", CYAN, RESET);
        for svc in &result.services {
            println!("{}│    - Port {}: {}{}", CYAN, svc.port, svc.service, RESET);
        }
    }

    if !result.vulnerabilities.is_empty() {
        println!("{}│  Vulnerabilities:{}{}", CYAN, RESET);
        for vuln in &result.vulnerabilities {
            let color = match vuln.severity {
                scanner::Severity::Critical => RED,
                scanner::Severity::High => "\x1b[91m",  // Bright red
                scanner::Severity::Medium => YELLOW,
                scanner::Severity::Low => "\x1b[34m",   // Blue
                scanner::Severity::Info => RESET,
            };
            print!("{}│    [{}] {}", CYAN, color, vuln.severity);
            if let Some(cve) = &vuln.cve_id {
                print!(" {}", cve);
            }
            println!(" (port {}){}", vuln.affected_port, RESET);
            println!("{}│      {}{}", CYAN, vuln.description, RESET);
        }
    }

    println!("{}│{}", CYAN, RESET);
    println!("{}└{}[Done]{}", CYAN, "─".repeat(40), RESET);

    // AI analysis if enabled
    if ai_analysis {
        println!("\n{}┌─ AI Analysis{}", CYAN, RESET);
        println!("{}│{}", CYAN, RESET);

        let settings = Settings::load()?;

        // Try to use cached token if no key provided
        let ai_key = if settings.ai_key.is_some() {
            settings.ai_key.clone()
        } else if let Some(database) = db {
            // Try to get best cached token
            if let Ok(token_mgr) = Ok(db::TokenManager::new(database.connection())) {
                if let Ok(Some(cached)) = token_mgr.get_best_token(settings.ai_provider.as_deref().unwrap_or("openai")) {
                    info!("Using cached token: {}...", cached.token_prefix);
                    // Note: We can't use the actual token, only the hash
                    // For security, we require the user to provide the actual token
                    // The cache just tracks which tokens worked well
                    None
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        let provider = ai::create_provider(ai_key.or(settings.ai_key), settings.ai_provider);
        let analysis = provider.analyze(result).await;

        // If AI call succeeded and we have the actual key, save to DB
        if let Some(key) = settings.ai_key {
            if let Some(database) = db {
                if let Ok(token_mgr) = Ok(db::TokenManager::new(database.connection())) {
                    let provider_name = settings.ai_provider.as_deref().unwrap_or("openai");
                    let _ = token_mgr.save_successful_token(provider_name, &key);
                    info!("Saved successful token to cache");
                }
            }
        }

        match analysis {
            Ok(report) => {
                // Print AI report with line-by-line formatting
                for line in report.lines() {
                    println!("{}│  {}{}", CYAN, line, RESET);
                }
            }
            Err(e) => {
                println!("{}│  {}Warning: AI analysis failed: {}{}", CYAN, YELLOW, e, RESET);
            }
        }
        println!("{}│{}", CYAN, RESET);
        println!("{}└{}[Done]{}", CYAN, "─".repeat(40), RESET);
    }

    Ok(())
}

async fn async_auto_scan(interface: Option<&str>, ai_analysis: bool) -> anyhow::Result<()> {
    println!("{}", ascii::auto_scan_banner());

    if let Some(iface) = interface {
        println!("{}~[~] Using interface: {}{}", YELLOW, iface, RESET);
    } else {
        println!("{}*[*] Auto-detecting network interface...{}", CYAN, RESET);
    }

    let config = scanner::ScanConfig {
        target: "auto".to_string(),
        ports: "1-1024".to_string(),
        timeout_ms: 3000,
        concurrent: 50,
    };

    let results = scanner::AutoScanner.scan_network(interface.map(|s| s.to_string()), &config).await;

    // Print summary
    println!("\n{}┌─ Auto-Scan Results{}", CYAN, RESET);
    println!("{}│{}", CYAN, RESET);
    println!("{}│  Hosts scanned: {}{}", CYAN, results.len(), RESET);

    let total_vulns: usize = results.iter().map(|r| r.vulnerabilities.len()).sum();
    println!("{}│  Total vulnerabilities found: {}{}", CYAN, total_vulns, RESET);

    for result in &results {
        if !result.vulnerabilities.is_empty() {
            println!("{}│{}", CYAN, RESET);
            println!("{}│  Host: {}{}", CYAN, result.target, RESET);
            for vuln in &result.vulnerabilities {
                let color = match vuln.severity {
                    scanner::Severity::Critical => RED,
                    scanner::Severity::High => "\x1b[91m",
                    scanner::Severity::Medium => YELLOW,
                    scanner::Severity::Low => "\x1b[34m",
                    scanner::Severity::Info => RESET,
                };
                print!("{}│    [{}] {}", CYAN, color, vuln.severity);
                if let Some(cve) = &vuln.cve_id {
                    print!(" {}", cve);
                }
                println!(" (port {}){}", vuln.affected_port, RESET);
            }
        }
    }

    println!("{}│{}", CYAN, RESET);
    println!("{}└{}[Done]{}", CYAN, "─".repeat(40), RESET);

    Ok(())
}

async fn async_map_hidden(range: &str, deep: bool) -> anyhow::Result<()> {
    println!("{}", ascii::map_hidden_banner());

    println!("{}~[~] Mapping hidden IPs in range: {}{}", YELLOW, range, RESET);
    if deep {
        println!("{}*[*] Deep scan mode enabled{}", CYAN, RESET);
    }

    let config = scanner::ScanConfig {
        target: range.to_string(),
        ports: if deep { "1-10000".to_string() } else { "22,23,80,443,8022,9022,2323,3389".to_string() },
        timeout_ms: 3000,
        concurrent: 50,
    };

    let results = scanner::HiddenMapper::map_hidden(range, deep, &config).await;

    // Print summary
    println!("\n{}┌─ Hidden IP Map Results{}", CYAN, RESET);
    println!("{}│{}", CYAN, RESET);
    println!("{}│  Hidden IPs discovered: {}{}", CYAN, results.len(), RESET);

    for result in &results {
        println!("{}│{}", CYAN, RESET);
        println!("{}│  {} - {}{}", CYAN, result.target, result.open_ports.len(), " ports", RESET);
        for vuln in &result.vulnerabilities {
            let color = match vuln.severity {
                scanner::Severity::Critical => RED,
                scanner::Severity::High => "\x1b[91m",
                scanner::Severity::Medium => YELLOW,
                scanner::Severity::Low => "\x1b[34m",
                scanner::Severity::Info => RESET,
            };
            print!("{}│    [{}] {}", CYAN, color, vuln.severity);
            if let Some(cve) = &vuln.cve_id {
                print!(" {}", cve);
            }
            println!(" (port {}){}", vuln.affected_port, RESET);
            println!("{}│      {}{}", CYAN, vuln.description, RESET);
        }
    }

    println!("{}│{}", CYAN, RESET);
    println!("{}└{}[Done]{}", CYAN, "─".repeat(40), RESET);

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
        println!("\n{}┌─ Kody Configuration{}", CYAN, RESET);
        println!("{}│{}", CYAN, RESET);
        println!("{}│  AI Provider: {}{}", CYAN, settings.ai_provider.as_deref().unwrap_or("not set"), RESET);
        println!("{}│  AI Key: {}{}", CYAN, if settings.ai_key.is_some() { "****" } else { "not set" }, RESET);
        println!("{}│  Output: {}{}", CYAN, settings.output_format.as_deref().unwrap_or("text"), RESET);

        // Show DB status
        if let Some(database) = db {
            let path = get_db_path();
            println!("{}│  Database: {}{}", CYAN, path.display(), RESET);

            // Get DB stats
            if let Ok(vuln_cache) = db::VulnCache::new(database.connection()) {
                if let Ok(count) = vuln_cache.count() {
                    println!("{}│    - Cached vulnerabilities: {}{}", CYAN, count, RESET);
                }
            }
            if let Ok(token_mgr) = db::TokenManager::new(database.connection()) {
                if let Ok(tokens) = token_mgr.get_all_tokens(settings.ai_provider.as_deref().unwrap_or("openai")) {
                    println!("{}│    - Cached tokens: {}{}", CYAN, tokens.len(), RESET);
                }
            }
        } else {
            println!("{}│  Database: not initialized (offline mode){}", CYAN, RESET);
        }

        println!("{}│{}", CYAN, RESET);
        println!("{}└{}[Done]{}", CYAN, "─".repeat(30), RESET);
        return Ok(());
    }

    let mut changed = false;

    if let Some(provider) = ai_provider {
        settings.ai_provider = Some(provider.clone());
        println!("{}+[+] AI provider set to: {}{}", GREEN, provider, RESET);
        changed = true;
    }

    if let Some(key) = ai_key {
        settings.ai_key = Some(key);
        println!("{}[+] AI API key configured{}", GREEN, RESET);
        changed = true;
    }

    if let Some(fmt) = output {
        settings.output_format = Some(fmt.clone());
        println!("{}[+] Output format set to: {}{}", GREEN, fmt, RESET);
        changed = true;
    }

    if changed {
        settings.save()?;
        println!("{}*[*] Configuration saved to ~/.kody/config.toml{}", CYAN, RESET);
    }

    Ok(())
}