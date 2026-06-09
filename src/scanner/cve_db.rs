//! Curated, version-aware vulnerability database.
//!
//! Two kinds of findings, kept strictly separate so output stays honest:
//!
//! 1. **CVE matches** — only emitted when we detected a real product+version
//!    and that version falls inside the documented vulnerable range. Every
//!    entry here is a real, correctly-attributed CVE.
//! 2. **Exposure findings** — best-practice / risk findings for services that
//!    are sensitive to expose (cleartext protocols, databases, remote access).
//!    These carry NO CVE id; they describe a risk, not a confirmed exploit.

use std::cmp::Ordering;
use super::vuln_rules::{Severity, Vulnerability};

/// A single curated CVE rule, matched against a detected product + version.
struct CveRule {
    /// Lowercase substring that must appear in the detected product name.
    product: &'static str,
    /// Vulnerable if detected version >= this (inclusive). `None` = no lower bound.
    introduced: Option<&'static str>,
    /// Vulnerable if detected version < this (exclusive, i.e. the fixed version).
    /// `None` = no upper bound.
    fixed: Option<&'static str>,
    cve: &'static str,
    severity: Severity,
    description: &'static str,
}

/// The curated rule set. Small on purpose: every entry is verified.
const CVE_RULES: &[CveRule] = &[
    CveRule {
        product: "openssh",
        introduced: Some("8.5p1"),
        fixed: Some("9.8p1"),
        cve: "CVE-2024-6387",
        severity: Severity::Critical,
        description: "regreSSHion: unauthenticated RCE in OpenSSH sshd (8.5p1-9.7p1)",
    },
    CveRule {
        product: "openssh",
        introduced: None,
        fixed: Some("9.3p2"),
        cve: "CVE-2023-38408",
        severity: Severity::High,
        description: "RCE via forwarded ssh-agent PKCS#11 provider (fixed in 9.3p2)",
    },
    CveRule {
        product: "apache",
        introduced: Some("2.4.49"),
        fixed: Some("2.4.51"),
        cve: "CVE-2021-41773",
        severity: Severity::Critical,
        description: "Apache httpd path traversal / RCE (2.4.49-2.4.50)",
    },
    CveRule {
        product: "nginx",
        introduced: Some("0.6.18"),
        fixed: Some("1.21.0"),
        cve: "CVE-2021-23017",
        severity: Severity::High,
        description: "nginx DNS resolver off-by-one heap write (before 1.21.0)",
    },
    CveRule {
        product: "vsftpd",
        introduced: Some("2.3.4"),
        fixed: Some("2.3.5"),
        cve: "CVE-2011-2523",
        severity: Severity::Critical,
        description: "vsftpd 2.3.4 contains a backdoor giving a root shell",
    },
    CveRule {
        product: "proftpd",
        introduced: None,
        fixed: Some("1.3.6"),
        cve: "CVE-2019-12815",
        severity: Severity::Critical,
        description: "ProFTPD mod_copy arbitrary file copy / RCE (before 1.3.6)",
    },
    CveRule {
        product: "exim",
        introduced: Some("4.87"),
        fixed: Some("4.92"),
        cve: "CVE-2019-10149",
        severity: Severity::Critical,
        description: "Exim 'The Return of the WIZard' RCE (4.87-4.91)",
    },
    CveRule {
        product: "samba",
        introduced: Some("3.5.0"),
        fixed: Some("4.6.4"),
        cve: "CVE-2017-7494",
        severity: Severity::Critical,
        description: "SambaCry: remote code execution via writable share (3.5.0-4.6.3)",
    },
];

/// Number of curated CVE rules currently loaded.
pub fn rule_count() -> usize {
    CVE_RULES.len()
}

/// Match a detected (product, version) against the curated CVE rules.
/// Returns only CVEs whose vulnerable range contains the detected version.
pub fn match_cves(product: &str, version: &str, port: u16) -> Vec<Vulnerability> {
    let product_lc = product.to_ascii_lowercase();
    let mut out = Vec::new();

    for rule in CVE_RULES {
        if !product_lc.contains(rule.product) {
            continue;
        }
        let above_floor = rule
            .introduced
            .map(|lo| cmp_version(version, lo) != Ordering::Less)
            .unwrap_or(true);
        let below_fix = rule
            .fixed
            .map(|hi| cmp_version(version, hi) == Ordering::Less)
            .unwrap_or(true);

        if above_floor && below_fix {
            out.push(Vulnerability::new(
                Some(rule.cve),
                rule.description,
                rule.severity,
                port,
            ));
        }
    }

    out
}

/// Exposure / best-practice findings for sensitive services. NEVER carries a
/// CVE id — these describe risk, not a confirmed vulnerability.
pub fn exposure_findings(port: u16) -> Vec<Vulnerability> {
    let mut v = Vec::new();
    let mut add = |desc: &str, sev: Severity| v.push(Vulnerability::new(None, desc, sev, port));

    match port {
        23 | 2323 => add(
            "Telnet exposed: credentials and data travel in cleartext. Use SSH.",
            Severity::High,
        ),
        21 => add(
            "FTP exposed: control channel is cleartext and may allow anonymous login. Prefer SFTP/FTPS.",
            Severity::Medium,
        ),
        6379 => add(
            "Redis exposed to the network: defaults to NO authentication. Bind to localhost or require a password.",
            Severity::Critical,
        ),
        27017 | 27018 => add(
            "MongoDB exposed to the network: verify authentication is enabled and access is firewalled.",
            Severity::Critical,
        ),
        11211 => add(
            "Memcached exposed: no authentication by design and usable for DDoS amplification. Firewall it.",
            Severity::Critical,
        ),
        9200 | 9300 => add(
            "Elasticsearch exposed: verify auth (X-Pack/security) is enabled; open clusters leak all data.",
            Severity::High,
        ),
        3306 => add(
            "MySQL/MariaDB reachable from the network: restrict to trusted hosts via firewall.",
            Severity::High,
        ),
        5432 => add(
            "PostgreSQL reachable from the network: restrict access and require strong authentication.",
            Severity::High,
        ),
        1433 => add(
            "Microsoft SQL Server reachable from the network: restrict access via firewall.",
            Severity::High,
        ),
        5984 => add(
            "CouchDB exposed: verify admin credentials are set (avoid 'admin party' mode).",
            Severity::High,
        ),
        3389 => add(
            "RDP exposed to the network: a top ransomware entry point. Put it behind a VPN and enable NLA.",
            Severity::High,
        ),
        5900 | 5901 => add(
            "VNC exposed: ensure a strong password is set; many VNC servers allow weak/no auth.",
            Severity::High,
        ),
        445 | 139 => add(
            "SMB/NetBIOS exposed: limit to trusted networks; a frequent lateral-movement vector.",
            Severity::Medium,
        ),
        80 | 8080 | 8000 | 8008 | 8888 => add(
            "Plain HTTP (no TLS): traffic is unencrypted. Redirect to HTTPS.",
            Severity::Low,
        ),
        25 => add(
            "SMTP exposed: verify it is not an open relay and that STARTTLS is enforced.",
            Severity::Info,
        ),
        _ => {}
    }

    v
}

/// Compare two version strings like "9.7p1", "2.4.52", "1.3.5b".
fn cmp_version(a: &str, b: &str) -> Ordering {
    let (a_base, a_p) = split_version(a);
    let (b_base, b_p) = split_version(b);

    let max_len = a_base.len().max(b_base.len());
    for i in 0..max_len {
        let av = a_base.get(i).copied().unwrap_or(0);
        let bv = b_base.get(i).copied().unwrap_or(0);
        match av.cmp(&bv) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    a_p.cmp(&b_p)
}

/// Split "9.7p1" into ([9, 7], 1). Trailing non-digits in a component are
/// ignored (e.g. "1.3.5b" -> ([1,3,5], 0)).
fn split_version(s: &str) -> (Vec<u64>, u64) {
    let (base, p) = match s.split_once('p') {
        Some((b, p)) if p.chars().next().is_some_and(|c| c.is_ascii_digit()) => {
            (b, leading_number(p))
        }
        _ => (s, 0),
    };
    let parts = base.split('.').map(leading_number).collect();
    (parts, p)
}

/// Parse the leading run of digits in a string into a number (0 if none).
fn leading_number(s: &str) -> u64 {
    let digits: String = s.chars().take_while(|c| c.is_ascii_digit()).collect();
    digits.parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmp_version_basic() {
        assert_eq!(cmp_version("1.18.0", "1.21.0"), Ordering::Less);
        assert_eq!(cmp_version("2.4.52", "2.4.49"), Ordering::Greater);
        assert_eq!(cmp_version("2.4.49", "2.4.49"), Ordering::Equal);
    }

    #[test]
    fn test_cmp_version_patch_suffix() {
        assert_eq!(cmp_version("9.7p1", "9.8p1"), Ordering::Less);
        assert_eq!(cmp_version("9.3p1", "9.3p2"), Ordering::Less);
        assert_eq!(cmp_version("9.3p2", "9.3p2"), Ordering::Equal);
    }

    #[test]
    fn test_openssh_vulnerable() {
        let v = match_cves("OpenSSH", "9.7p1", 22);
        assert!(v.iter().any(|x| x.cve_id.as_deref() == Some("CVE-2024-6387")));
    }

    #[test]
    fn test_openssh_patched() {
        // 9.8p1 is fixed for regreSSHion and 9.3p2 for the agent bug.
        let v = match_cves("OpenSSH", "9.8p1", 22);
        assert!(v.is_empty(), "patched OpenSSH should report no CVEs, got {:?}", v);
    }

    #[test]
    fn test_nginx_old_vulnerable() {
        let v = match_cves("nginx", "1.18.0", 80);
        assert!(v.iter().any(|x| x.cve_id.as_deref() == Some("CVE-2021-23017")));
    }

    #[test]
    fn test_nginx_new_clean() {
        let v = match_cves("nginx", "1.25.0", 80);
        assert!(v.is_empty());
    }

    #[test]
    fn test_vsftpd_backdoor_exact() {
        assert!(!match_cves("vsftpd", "2.3.4", 21).is_empty());
        assert!(match_cves("vsftpd", "3.0.3", 21).is_empty());
    }

    #[test]
    fn test_exposure_redis_critical() {
        let v = exposure_findings(6379);
        assert_eq!(v.len(), 1);
        assert!(v[0].cve_id.is_none());
        assert_eq!(v[0].severity, Severity::Critical);
    }

    #[test]
    fn test_no_exposure_for_random_port() {
        assert!(exposure_findings(12345).is_empty());
    }
}
