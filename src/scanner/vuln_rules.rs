use serde::{Deserialize, Serialize};

/// Severity levels for vulnerabilities
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Critical => write!(f, "CRITICAL"),
            Severity::High => write!(f, "HIGH"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::Low => write!(f, "LOW"),
            Severity::Info => write!(f, "INFO"),
        }
    }
}

/// Color code for severity (ANSI escape sequence)
impl Severity {
    pub fn color(&self) -> &'static str {
        match self {
            Severity::Critical => "\x1b[91m",  // Bright red
            Severity::High => "\x1b[31m",      // Red
            Severity::Medium => "\x1b[33m",    // Yellow
            Severity::Low => "\x1b[34m",       // Blue
            Severity::Info => "\x1b[37m",      // White/Gray
        }
    }

    pub fn reset(&self) -> &'static str {
        "\x1b[0m"
    }
}

/// A discovered vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub cve_id: Option<String>,
    pub description: String,
    pub severity: Severity,
    pub affected_port: u16,
    pub service: Option<String>,
}

impl Vulnerability {
    pub fn new(cve: Option<&str>, desc: &str, severity: Severity, port: u16) -> Self {
        Self {
            cve_id: cve.map(|s| s.to_string()),
            description: desc.to_string(),
            severity,
            affected_port: port,
            service: None,
        }
    }

    pub fn with_service(mut self, service: &str) -> Self {
        self.service = Some(service.to_string());
        self
    }
}

/// Match vulnerabilities based on port and service
pub fn match_rules(port: u16, service: Option<&str>) -> Vec<Vulnerability> {
    let mut vulns = Vec::new();

    // SSH vulnerabilities
    if port == 22 {
        vulns.push(Vulnerability::new(
            Some("CVE-2023-38408"),
            "OpenSSH before 9.3p1 has remote code execution vulnerability",
            Severity::Medium,
            port,
        ));
        if let Some(svc) = service {
            if svc.contains("OpenSSH 6.") || svc.contains("OpenSSH 7.") {
                vulns.push(Vulnerability::new(
                    Some("CVE-2021-28041"),
                    "OpenSSH < 8.8 vulnerable to remote code execution via pre-authentication",
                    Severity::Critical,
                    port,
                ));
            }
        }
    }

    // FTP vulnerabilities
    if port == 21 {
        vulns.push(Vulnerability::new(
            Some("CVE-2021-35562"),
            "FTP server allows anonymous access or cleartext authentication",
            Severity::Medium,
            port,
        ));
        vulns.push(Vulnerability::new(
            None,
            "FTP transmits data in cleartext - use SFTP instead",
            Severity::High,
            port,
        ));
    }

    // Telnet vulnerabilities
    if port == 23 {
        vulns.push(Vulnerability::new(
            None,
            "Telnet transmits all data (including passwords) in cleartext",
            Severity::Critical,
            port,
        ));
        vulns.push(Vulnerability::new(
            Some("CVE-2020-10188"),
            "Telnet protocol has multiple vulnerabilities allowing code execution",
            Severity::High,
            port,
        ));
    }

    // SMTP vulnerabilities
    if port == 25 {
        vulns.push(Vulnerability::new(
            None,
            "SMTP server detected - verify SPF/DKIM/DMARC configuration",
            Severity::Info,
            port,
        ));
        vulns.push(Vulnerability::new(
            Some("CVE-2020-12353"),
            "SMTP STARTTLS vulnerability allows MITM attacks",
            Severity::Medium,
            port,
        ));
    }

    // DNS zone transfer
    if port == 53 {
        vulns.push(Vulnerability::new(
            None,
            "DNS server detected - verify zone transfer restrictions",
            Severity::Medium,
            port,
        ));
    }

    // HTTP vulnerabilities
    if port == 80 || port == 443 || port == 8080 || port == 8443 {
        if let Some(svc) = service {
            if svc.contains("Apache") {
                if svc.contains("2.4.") && !svc.contains("2.4.55") {
                    vulns.push(Vulnerability::new(
                        Some("CVE-2023-25690"),
                        "Apache HTTPD < 2.4.55 allows HTTP request smuggling",
                        Severity::High,
                        port,
                    ));
                }
            }
            if svc.contains("nginx") {
                if svc.contains("1.18.") || svc.contains("1.19.") {
                    vulns.push(Vulnerability::new(
                        Some("CVE-2022-41741"),
                        "nginx 1.18.0-1.22.0 has multipart smuggling vulnerability",
                        Severity::Medium,
                        port,
                    ));
                }
            }
            if svc.contains("IIS") || svc.contains("Microsoft") {
                vulns.push(Vulnerability::new(
                    Some("CVE-2022-41040"),
                    "Microsoft IIS potential RCE via HTTP request",
                    Severity::High,
                    port,
                ));
            }
        }

        // Generic web server checks
        vulns.push(Vulnerability::new(
            None,
            "Web server detected - verify HTTP security headers (X-Frame-Options, CSP, etc)",
            Severity::Low,
            port,
        ));
    }

    // SMB vulnerabilities (Windows file sharing)
    if port == 445 {
        vulns.push(Vulnerability::new(
            Some("CVE-2022-37919"),
            "SMB signature bypass vulnerability in Windows",
            Severity::Critical,
            port,
        ));
        vulns.push(Vulnerability::new(
            Some("CVE-2020-0796"),
            "SMBGhost - remote code execution in Windows SMB",
            Severity::Critical,
            port,
        ));
    }

    // MySQL/MariaDB
    if port == 3306 {
        vulns.push(Vulnerability::new(
            Some("CVE-2022-27448"),
            "MySQL server exposed on network without firewall",
            Severity::Critical,
            port,
        ));
        vulns.push(Vulnerability::new(
            None,
            "MySQL database should not be directly accessible from network",
            Severity::Critical,
            port,
        ));
    }

    // PostgreSQL
    if port == 5432 {
        vulns.push(Vulnerability::new(
            Some("CVE-2024-1597"),
            "PostgreSQL SQL injection vulnerability in prepared statements",
            Severity::High,
            port,
        ));
        vulns.push(Vulnerability::new(
            None,
            "PostgreSQL should not be exposed without authentication",
            Severity::Critical,
            port,
        ));
    }

    // Redis
    if port == 6379 {
        vulns.push(Vulnerability::new(
            Some("CVE-2023-22458"),
            "Redis without authentication - critical risk",
            Severity::Critical,
            port,
        ));
        vulns.push(Vulnerability::new(
            None,
            "Redis allows full system access without authentication",
            Severity::Critical,
            port,
        ));
    }

    // MongoDB
    if port == 27017 {
        vulns.push(Vulnerability::new(
            Some("CVE-2023-0145"),
            "MongoDB without authentication allows full database access",
            Severity::Critical,
            port,
        ));
        vulns.push(Vulnerability::new(
            None,
            "MongoDB should require authentication - currently open",
            Severity::Critical,
            port,
        ));
    }

    // RDP (Windows remote desktop)
    if port == 3389 {
        vulns.push(Vulnerability::new(
            Some("CVE-2023-29339"),
            "RDP exposed to network is a primary attack vector",
            Severity::Critical,
            port,
        ));
        vulns.push(Vulnerability::new(
            None,
            "RDP should not be exposed to internet without VPN",
            Severity::High,
            port,
        ));
    }

    // VNC
    if port == 5900 || port == 5901 {
        vulns.push(Vulnerability::new(
            Some("CVE-2023-28354"),
            "VNC server detected - ensure password protection is enabled",
            Severity::High,
            port,
        ));
    }

    // LDAP
    if port == 389 || port == 636 {
        vulns.push(Vulnerability::new(
            None,
            "LDAP server detected - verify access controls",
            Severity::Medium,
            port,
        ));
    }

    // Rsync
    if port == 873 {
        vulns.push(Vulnerability::new(
            None,
            "Rsync daemon may allow unauthorized file access",
            Severity::High,
            port,
        ));
    }

    // Memcached
    if port == 11211 {
        vulns.push(Vulnerability::new(
            Some("CVE-2023-28435"),
            "Memcached without authentication allows data exposure",
            Severity::High,
            port,
        ));
    }

    vulns
}

/// Get service name from port number
pub fn get_service_name(port: u16) -> &'static str {
    match port {
        20 => "ftp-data",
        21 => "ftp",
        22 => "ssh",
        23 => "telnet",
        25 => "smtp",
        53 => "dns",
        67 => "dhcp",
        68 => "dhcp",
        69 => "tftp",
        80 => "http",
        110 => "pop3",
        119 => "nntp",
        123 => "ntp",
        135 => "msrpc",
        137 => "netbios-ns",
        138 => "netbios-dgm",
        139 => "netbios-ssn",
        143 => "imap",
        161 => "snmp",
        162 => "snmptrap",
        389 => "ldap",
        443 => "https",
        445 => "microsoft-ds",
        465 => "smtps",
        514 => "syslog",
        515 => "printer",
        543 => "postgresql",
        587 => "submission",
        636 => "ldaps",
        873 => "rsync",
        902 => "vmware",
        993 => "imaps",
        995 => "pop3s",
        1080 => "socks",
        1433 => "mssql",
        1434 => "mssql-m",
        1521 => "oracle",
        2049 => "nfs",
        3306 => "mysql",
        3389 => "ms-wbt",
        5432 => "postgresql",
        5900 => "vnc",
        5901 => "vnc-1",
        6379 => "redis",
        8080 => "http-proxy",
        8443 => "https-alt",
        8888 => "http-alt",
        9100 => "printer",
        9200 => "elasticsearch",
        27017 => "mongodb",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_color() {
        assert_eq!(Severity::Critical.color(), "\x1b[91m");
        assert_eq!(Severity::High.color(), "\x1b[31m");
        assert_eq!(Severity::Medium.color(), "\x1b[33m");
        assert_eq!(Severity::Low.color(), "\x1b[34m");
        assert_eq!(Severity::Info.color(), "\x1b[37m");
    }

    #[test]
    fn test_ssh_vulnerability() {
        let vulns = match_rules(22, Some("OpenSSH 7.4"));
        assert!(!vulns.is_empty());
        // Should have at least one critical vulnerability for old SSH
        assert!(vulns.iter().any(|v| v.severity == Severity::Critical));
    }

    #[test]
    fn test_mysql_vulnerability() {
        let vulns = match_rules(3306, None);
        assert!(!vulns.is_empty());
        assert!(vulns.iter().any(|v| v.severity == Severity::Critical));
    }

    #[test]
    fn test_redis_vulnerability() {
        let vulns = match_rules(6379, None);
        assert!(!vulns.is_empty());
        assert!(vulns.iter().any(|v| v.severity == Severity::Critical));
    }

    #[test]
    fn test_service_name() {
        assert_eq!(get_service_name(22), "ssh");
        assert_eq!(get_service_name(80), "http");
        assert_eq!(get_service_name(443), "https");
        assert_eq!(get_service_name(3306), "mysql");
    }

    #[test]
    fn test_unknown_port() {
        let vulns = match_rules(12345, None);
        // Unknown ports should not have automatic vulnerabilities
        assert!(vulns.is_empty());
    }
}