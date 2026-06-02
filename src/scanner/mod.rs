pub mod ports;
pub mod vuln_rules;

pub mod ip_scan;
pub mod auto_scan;
pub mod hidden_map;

pub use ports::{parse_port_range, tcp_scan, resolve_host};
pub use vuln_rules::{Severity, Vulnerability, match_rules, get_service_name};
pub use auto_scan::AutoScanner;
pub use hidden_map::HiddenMapper;

use serde::{Deserialize, Serialize};
use crate::ai::{ScanResult, ServiceInfo};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanConfig {
    pub target: String,
    pub ports: String,
    pub timeout_ms: u64,
    pub concurrent: usize,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            target: String::new(),
            ports: "1-1024".to_string(),
            timeout_ms: 3000,
            concurrent: 100,
        }
    }
}

/// The main Scanner struct that orchestrates port scanning and vulnerability detection
pub struct Scanner;

impl Scanner {
    /// Run a scan on the target with the given configuration
    pub async fn run(target: &str, ports: Vec<u16>, config: &ScanConfig) -> ScanResult {
        // Resolve hostname if needed
        let resolved_target = resolve_host(target).await.unwrap_or_else(|| target.to_string());

        // Scan ports concurrently
        let open_ports = tcp_scan(&resolved_target, ports, config.concurrent, config.timeout_ms).await;

        // Detect services
        let services: Vec<ServiceInfo> = open_ports.iter()
            .map(|port| ServiceInfo {
                port: *port,
                service: get_service_name(*port).to_string(),
                version: None,
            })
            .collect();

        // Match vulnerabilities
        let mut vulnerabilities = Vec::new();
        for service in &services {
            let vulns = match_rules(service.port, Some(&service.service));
            for mut vuln in vulns {
                vuln.service = Some(service.service.clone());
                vulnerabilities.push(vuln);
            }
        }

        ScanResult {
            target: target.to_string(),
            open_ports,
            services,
            vulnerabilities,
            raw_output: format!("Scan completed for {} - {} ports scanned", target, config.ports),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_port_range() {
        let ports = parse_port_range("80,443,8000-8010");
        assert!(ports.contains(&80));
        assert!(ports.contains(&443));
        assert!(ports.contains(&8000));
        assert!(ports.contains(&8010));
    }

    #[test]
    fn test_parse_single_port() {
        let ports = parse_port_range("8080");
        assert_eq!(ports, vec![8080]);
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Critical), "CRITICAL");
        assert_eq!(format!("{}", Severity::High), "HIGH");
        assert_eq!(format!("{}", Severity::Medium), "MEDIUM");
        assert_eq!(format!("{}", Severity::Low), "LOW");
        assert_eq!(format!("{}", Severity::Info), "INFO");
    }
}