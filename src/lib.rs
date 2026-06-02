//! Kody - CLI Vulnerability Scanner
//!
//! A Rust-based vulnerability scanner with AI integration and offline method cache.

pub mod ascii;
pub mod cli;
pub mod config;
pub mod ai;
pub mod scanner;
pub mod db;
pub mod network;

pub use ascii::banner;
pub use config::{Config, Settings};
pub use ai::{AiProvider, ScanResult, Vulnerability, Severity, create_provider};
pub use scanner::{AutoScanner, HiddenMapper, ScanConfig, parse_port_range};
pub use scanner::ip_scan::IpScanner;
pub use db::Database;
pub use db::tokens::CachedToken;
pub use db::methods::CachedMethod;
pub use db::vulns::CachedVuln;
pub use db::history::ScanRecord;
pub use network::{DiscoveredHost, ScanOptions, Subnet};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");