use async_trait::async_trait;
use crate::ai::{AiProvider, AiError, ScanResult, Vulnerability, Severity};

pub struct OfflineProvider;

impl OfflineProvider {
    pub fn new() -> Self {
        Self
    }

    fn analyze_vulnerabilities(&self, scan_result: &ScanResult) -> Vec<Vulnerability> {
        let mut vulns = Vec::new();

        // Pre-defined vulnerability rules based on common services/ports
        for service in &scan_result.services {
            // SSH vulnerabilities
            if service.port == 22 {
                if let Some(version) = &service.version {
                    if version.contains("OpenSSH 7.") || version.contains("OpenSSH 6.") {
                        vulns.push(Vulnerability {
                            cve_id: Some("CVE-2023-38408".to_string()),
                            description: "OpenSSH versions before 9.3 have known vulnerabilities".to_string(),
                            severity: Severity::Medium,
                            affected_port: service.port,
                            service: Some(service.service.clone()),
                        });
                    }
                }
            }

            // FTP vulnerabilities
            if service.port == 21 {
                vulns.push(Vulnerability {
                    cve_id: Some("CVE-2021-35562".to_string()),
                    description: "FTP servers may allow anonymous access or have cleartext authentication".to_string(),
                    severity: Severity::Medium,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }

            // SMTP vulnerabilities
            if service.port == 25 {
                vulns.push(Vulnerability {
                    cve_id: None,
                    description: "SMTP server detected - verify proper SPF/DKIM configuration".to_string(),
                    severity: Severity::Info,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }

            // HTTP vulnerabilities
            if service.port == 80 || service.port == 443 {
                if let Some(version) = &service.version {
                    if version.contains("Apache 2.4.") {
                        vulns.push(Vulnerability {
                            cve_id: Some("CVE-2023-25690".to_string()),
                            description: "Apache HTTPD prior to 2.4.55 has mod_proxy SSRF issues".to_string(),
                            severity: Severity::High,
                            affected_port: service.port,
                            service: Some(service.service.clone()),
                        });
                    }
                    if version.contains("nginx") && version.contains("1.18") {
                        vulns.push(Vulnerability {
                            cve_id: Some("CVE-2022-41741".to_string()),
                            description: "nginx 1.18.0 - 1.22.0 have multipart smuggling issues".to_string(),
                            severity: Severity::Medium,
                            affected_port: service.port,
                            service: Some(service.service.clone()),
                        });
                    }
                }

                vulns.push(Vulnerability {
                    cve_id: None,
                    description: format!("Web server on port {} - verify security headers", service.port),
                    severity: Severity::Low,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }

            // MySQL/MariaDB vulnerabilities
            if service.port == 3306 {
                vulns.push(Vulnerability {
                    cve_id: Some("CVE-2022-27448".to_string()),
                    description: "MySQL/MariaDB servers should not be exposed publicly".to_string(),
                    severity: Severity::Critical,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }

            // PostgreSQL vulnerabilities
            if service.port == 5432 {
                vulns.push(Vulnerability {
                    cve_id: Some("CVE-2024-1597".to_string()),
                    description: "PostgreSQL SQL injection vulnerability in prepared statements".to_string(),
                    severity: Severity::High,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }

            // Redis vulnerabilities
            if service.port == 6379 {
                vulns.push(Vulnerability {
                    cve_id: Some("CVE-2023-22458".to_string()),
                    description: "Redis servers without authentication are critical risk".to_string(),
                    severity: Severity::Critical,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }

            // MongoDB vulnerabilities
            if service.port == 27017 {
                vulns.push(Vulnerability {
                    cve_id: Some("CVE-2023-0145".to_string()),
                    description: "MongoDB without authentication allows full database access".to_string(),
                    severity: Severity::Critical,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }

            // RDP vulnerabilities
            if service.port == 3389 {
                vulns.push(Vulnerability {
                    cve_id: Some("CVE-2023-29339".to_string()),
                    description: "RDP exposed to internet is a primary attack vector".to_string(),
                    severity: Severity::Critical,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }

            // SMB vulnerabilities (common on Windows)
            if service.port == 445 {
                vulns.push(Vulnerability {
                    cve_id: Some("CVE-2022-37919".to_string()),
                    description: "SMB signature bypass vulnerability in Windows".to_string(),
                    severity: Severity::Critical,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }

            // Telnet (unencrypted)
            if service.port == 23 {
                vulns.push(Vulnerability {
                    cve_id: None,
                    description: "Telnet transmits data in cleartext - use SSH instead".to_string(),
                    severity: Severity::High,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }

            // VNC vulnerabilities
            if service.port == 5900 {
                vulns.push(Vulnerability {
                    cve_id: Some("CVE-2023-28354".to_string()),
                    description: "VNC servers should have password protection enabled".to_string(),
                    severity: Severity::High,
                    affected_port: service.port,
                    service: Some(service.service.clone()),
                });
            }
        }

        vulns
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
        let vulns = self.analyze_vulnerabilities(&scan_result);
        Ok(self.generate_report(&scan_result, &vulns))
    }

    fn name(&self) -> &str {
        "offline"
    }

    fn is_configured(&self) -> bool {
        true // Offline mode is always available
    }
}