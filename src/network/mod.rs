//! Network discovery module for kody
//! Provides network interface detection, ARP scanning, and IP discovery

pub mod discovery;

pub use discovery::{Subnet, discover_hosts, get_local_subnet};

/// Result of scanning a single host
#[derive(Debug, Clone)]
pub struct DiscoveredHost {
    pub ip: String,
    #[allow(dead_code)]
    pub mac: Option<String>,
    #[allow(dead_code)]
    pub hostname: Option<String>,
    #[allow(dead_code)]
    pub ports: Vec<u16>,
    #[allow(dead_code)]
    pub is_alive: bool,
}

/// Scan configuration
#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub timeout_ms: u64,
    pub concurrent: usize,
    #[allow(dead_code)]
    pub ping_first: bool,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            timeout_ms: 2000,
            concurrent: 50,
            ping_first: true,
        }
    }
}