use serde::{Deserialize, Serialize};

/// Severity levels for vulnerabilities.
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
        let s = match self {
            Severity::Critical => "CRITICAL",
            Severity::High => "HIGH",
            Severity::Medium => "MEDIUM",
            Severity::Low => "LOW",
            Severity::Info => "INFO",
        };
        write!(f, "{}", s)
    }
}

impl Severity {
    /// Rank used for sorting (most severe first).
    pub fn rank(&self) -> u8 {
        match self {
            Severity::Critical => 0,
            Severity::High => 1,
            Severity::Medium => 2,
            Severity::Low => 3,
            Severity::Info => 4,
        }
    }

    /// ANSI color for this severity.
    #[allow(dead_code)]
    pub fn color(&self) -> &'static str {
        match self {
            Severity::Critical => "\x1b[91m", // Bright red
            Severity::High => "\x1b[31m",      // Red
            Severity::Medium => "\x1b[33m",    // Yellow
            Severity::Low => "\x1b[34m",       // Blue
            Severity::Info => "\x1b[37m",      // White/Gray
        }
    }
}

/// A discovered vulnerability or exposure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub cve_id: Option<String>,
    pub description: String,
    pub severity: Severity,
    pub affected_port: u16,
    pub service: Option<String>,
    /// What an attacker could achieve — "qué se podría dañar".
    #[serde(default)]
    pub impact: Option<String>,
    /// Where it lives / can be hit — "dónde" (endpoint, header, service).
    #[serde(default)]
    pub location: Option<String>,
}

impl Vulnerability {
    pub fn new(cve: Option<&str>, desc: &str, severity: Severity, port: u16) -> Self {
        Self {
            cve_id: cve.map(|s| s.to_string()),
            description: desc.to_string(),
            severity,
            affected_port: port,
            service: None,
            impact: None,
            location: None,
        }
    }

    pub fn with_service(mut self, service: &str) -> Self {
        self.service = Some(service.to_string());
        self
    }

    /// Set the attacker-impact narrative ("qué podría dañar").
    pub fn with_impact(mut self, impact: &str) -> Self {
        self.impact = Some(impact.to_string());
        self
    }

    /// Set where the issue can be reached ("dónde").
    pub fn with_location(mut self, location: &str) -> Self {
        self.location = Some(location.to_string());
        self
    }

    /// True if this is a confirmed CVE match (vs. a best-practice exposure).
    #[allow(dead_code)]
    pub fn is_cve(&self) -> bool {
        self.cve_id.is_some()
    }
}

/// Map a well-known port number to a service name.
pub fn get_service_name(port: u16) -> &'static str {
    match port {
        20 => "ftp-data",
        21 => "ftp",
        22 => "ssh",
        23 => "telnet",
        25 => "smtp",
        53 => "dns",
        67 | 68 => "dhcp",
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
        2375 | 2376 => "docker",
        3000 => "http-dev",
        3306 => "mysql",
        3389 => "ms-wbt",
        5432 => "postgresql",
        5601 => "kibana",
        5900 => "vnc",
        5901 => "vnc-1",
        5984 => "couchdb",
        6379 => "redis",
        6443 => "kube-api",
        8080 => "http-proxy",
        8443 => "https-alt",
        8888 => "http-alt",
        9100 => "printer",
        9200 | 9300 => "elasticsearch",
        11211 => "memcached",
        15672 => "rabbitmq-mgmt",
        27017 | 27018 => "mongodb",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_severity_display() {
        assert_eq!(format!("{}", Severity::Critical), "CRITICAL");
        assert_eq!(format!("{}", Severity::Info), "INFO");
    }

    #[test]
    fn test_severity_rank_order() {
        assert!(Severity::Critical.rank() < Severity::High.rank());
        assert!(Severity::High.rank() < Severity::Info.rank());
    }

    #[test]
    fn test_service_name() {
        assert_eq!(get_service_name(22), "ssh");
        assert_eq!(get_service_name(80), "http");
        assert_eq!(get_service_name(3306), "mysql");
        assert_eq!(get_service_name(12345), "unknown");
    }
}
