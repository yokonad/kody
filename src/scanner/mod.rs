pub mod ports;
pub mod vuln_rules;
pub mod banner;
pub mod web;
pub mod cve_db;

pub mod ip_scan;
pub mod auto_scan;
pub mod hidden_map;

pub use ports::{parse_port_range, parse_port_spec, tcp_scan, resolve_host, resolve_all, is_ip_literal, TOP_PORTS};
pub use vuln_rules::{Severity, Vulnerability, get_service_name};
pub use banner::grab_fingerprint;
#[allow(unused_imports)]
pub use banner::Fingerprint;
pub use cve_db::{match_cves, exposure_findings};

/// Total "TTPs loaded" — curated CVE rules + the common-port probe set.
/// Used for the session banner's TTPs count.
pub fn ttp_count() -> usize {
    cve_db::rule_count() + TOP_PORTS.len()
}
pub use auto_scan::AutoScanner;
pub use hidden_map::HiddenMapper;

use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use crate::ai::{ScanResult, ServiceInfo};

/// Assess a single open port: fingerprint it, then collect real CVE matches
/// (only when a version was detected) plus honest exposure findings.
pub async fn assess_port(target: &str, port: u16, timeout_ms: u64) -> (ServiceInfo, Vec<Vulnerability>) {
    // Web ports get a full HTTP recon (fingerprint + security-header findings);
    // everything else just reads the connect banner.
    let (fp, web_findings) = match banner::web_scheme(port) {
        Some(tls) => web::analyze_web(target, port, tls, timeout_ms).await,
        None => (grab_fingerprint(target, port, timeout_ms).await, Vec::new()),
    };

    // Prefer the detected product as the service label; fall back to the port map.
    let service_label = fp
        .display()
        .unwrap_or_else(|| get_service_name(port).to_string());

    let service_info = ServiceInfo {
        port,
        service: service_label.clone(),
        version: fp.version.clone(),
    };

    let mut vulns = Vec::new();

    // Real CVE matching requires both a product and a version.
    if let (Some(product), Some(version)) = (fp.product.as_deref(), fp.version.as_deref()) {
        for v in match_cves(product, version, port) {
            vulns.push(v.with_service(&service_label));
        }
    }

    // HTTP header / hygiene findings (web ports only).
    for v in web_findings {
        vulns.push(v.with_service(&service_label));
    }

    // Exposure findings apply regardless of version.
    for v in exposure_findings(port) {
        vulns.push(v.with_service(get_service_name(port)));
    }

    (service_info, vulns)
}

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
    /// Run a scan on the target with the given configuration.
    pub async fn run(target: &str, ports: Vec<u16>, config: &ScanConfig) -> ScanResult {
        // Resolve hostname if needed.
        let resolved_target = resolve_host(target).await.unwrap_or_else(|| target.to_string());

        // Scan ports concurrently.
        let num_ports = ports.len();
        let open_ports = tcp_scan(&resolved_target, ports, config.concurrent, config.timeout_ms).await;

        // Fingerprint each open port concurrently and assess it.
        let fp_concurrency = config.concurrent.min(open_ports.len().max(1));
        let assessed: Vec<(ServiceInfo, Vec<Vulnerability>)> = stream::iter(open_ports.clone())
            .map(|port| {
                let t = resolved_target.clone();
                async move { assess_port(&t, port, config.timeout_ms).await }
            })
            .buffer_unordered(fp_concurrency.max(1))
            .collect()
            .await;

        let mut services = Vec::with_capacity(assessed.len());
        let mut vulnerabilities = Vec::new();
        for (svc, vulns) in assessed {
            services.push(svc);
            vulnerabilities.extend(vulns);
        }

        // Stable, useful ordering: services by port, findings by severity.
        services.sort_by_key(|s| s.port);
        vulnerabilities.sort_by(|a, b| {
            a.severity
                .rank()
                .cmp(&b.severity.rank())
                .then(a.affected_port.cmp(&b.affected_port))
        });

        ScanResult {
            target: target.to_string(),
            open_ports,
            services,
            vulnerabilities,
            raw_output: format!("Scan completed for {} - {} ports scanned", target, num_ports),
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