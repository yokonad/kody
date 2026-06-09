use async_trait::async_trait;
use crate::ai::{AiProvider, AiError, ScanResult, Vulnerability, Severity};

pub struct OfflineProvider;

impl OfflineProvider {
    pub fn new() -> Self {
        Self
    }

    fn generate_report(&self, scan_result: &ScanResult, vulns: &[Vulnerability]) -> String {
        let mut report = String::new();
        report.push_str("=== KODY OFFLINE VULNERABILITY ANALYSIS ===\n\n");
        report.push_str(&format!("Target: {}\n", scan_result.target));
        report.push_str(&format!("Ports scanned: {}\n", scan_result.open_ports.len()));
        report.push_str(&format!("Services detected: {}\n\n", scan_result.services.len()));

        // Summary by severity
        let critical = vulns.iter().filter(|v| v.severity == Severity::Critical).count();
        let high = vulns.iter().filter(|v| v.severity == Severity::High).count();
        let medium = vulns.iter().filter(|v| v.severity == Severity::Medium).count();
        let low = vulns.iter().filter(|v| v.severity == Severity::Low).count();
        let info = vulns.iter().filter(|v| v.severity == Severity::Info).count();

        report.push_str("Vulnerability Summary:\n");
        report.push_str(&format!("  Critical: {}\n", critical));
        report.push_str(&format!("  High:     {}\n", high));
        report.push_str(&format!("  Medium:   {}\n", medium));
        report.push_str(&format!("  Low:      {}\n", low));
        report.push_str(&format!("  Info:     {}\n\n", info));

        if !vulns.is_empty() {
            report.push_str("Detailed Findings:\n");
            report.push_str(&"─".repeat(50));
            report.push('\n');

            for vuln in vulns {
                report.push_str(&format!("[{}] ", vuln.severity));
                if let Some(cve) = &vuln.cve_id {
                    report.push_str(&format!("{} - ", cve));
                }
                report.push_str(&format!("Port {}\n", vuln.affected_port));
                report.push_str(&format!("  {}\n\n", vuln.description));
            }
        } else {
            report.push_str("No vulnerabilities detected based on known patterns.\n");
        }

        report.push_str("\nRecommendation: Use AI analysis for detailed CVE information.\n");
        report.push_str("Configure with: kody config --ai-key <your-key>\n");

        report
    }
}

#[async_trait]
impl AiProvider for OfflineProvider {
    async fn analyze(&self, scan_result: ScanResult) -> Result<String, AiError> {
        // Report the findings the scanner already produced (real, version-aware
        // CVE matches + exposures) instead of re-deriving a second, weaker set.
        let vulns = scan_result.vulnerabilities.clone();
        Ok(self.generate_report(&scan_result, &vulns))
    }

    fn name(&self) -> &str {
        "offline"
    }

    fn is_configured(&self) -> bool {
        true // Offline mode is always available
    }
}