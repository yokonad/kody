use crate::scanner::{ScanResult, ServiceInfo, Vulnerability, Severity, ScanConfig, parse_port_range};

pub struct IpScanner;

impl IpScanner {
    /// Scan a specific IP or domain for vulnerabilities
    pub async fn scan(target: &str, ports: &str, config: &ScanConfig) -> ScanResult {
        let port_list = parse_port_range(ports);

        // Use the new Scanner::run which does actual TCP scanning
        // For Phase 2, this calls the actual TCP scanning implementation
        crate::scanner::Scanner::run(target, port_list, config).await
    }
}