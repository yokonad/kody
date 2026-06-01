//! Network discovery module for kody
//! Provides network interface detection, ARP scanning, and IP discovery

pub mod discovery;

pub use discovery::{NetworkInterface, Subnet, discover_hosts, get_local_subnet};

/// Result of scanning a single host
#[derive(Debug, Clone)]
pub struct DiscoveredHost {
    pub ip: String,
    pub mac: Option<String>,
    pub hostname: Option<String>,
    pub ports: Vec<u16>,
    pub is_alive: bool,
}

/// Scan configuration
#[derive(Debug, Clone)]
pub struct ScanOptions {
    pub timeout_ms: u64,
    pub concurrent: usize,
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