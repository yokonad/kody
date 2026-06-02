use crate::scanner::{ScanResult, ScanConfig, parse_port_range};

#[allow(dead_code)]
pub struct IpScanner;

impl IpScanner {
    /// Scan a specific IP or domain for vulnerabilities
    #[allow(dead_code)]
    pub async fn scan(target: &str, ports: &str, config: &ScanConfig) -> ScanResult {
        let port_list = parse_port_range(ports);

        // Use the new Scanner::run which does actual TCP scanning
        // For Phase 2, this calls the actual TCP scanning implementation
        crate::scanner::Scanner::run(target, port_list, config).await
    }
}