use crate::ai::{ScanResult, ServiceInfo, Vulnerability, Severity};
use crate::scanner::{ScanConfig, parse_port_range, get_service_name, match_rules};
use crate::network::{self, Subnet};

pub struct HiddenMapper;

/// Non-standard ports that might indicate "hidden" or stealth services
const STEALTH_PORTS: &[u16] = &[
    22,     // SSH (normal)
    23,     // Telnet (often blocked, alternative: 2222, 8022)
    80,     // HTTP (normal)
    443,    // HTTPS (normal)
    1022,   // Alternative SSH
    1023,   // Alternative Telnet
    2222,   // Alternative SSH
    2323,   // Alternative Telnet
    3022,   // Alternative SSH
    3306,   // MySQL (normal)
    3389,   // RDP (normal, alternative: 53389)
    4444,   // Metasploit default
    5000,   // Python Flask default
    5555,   // Android debug bridge
    5900,   // VNC (normal)
    5901,   // VNC :1
    6022,   // Alternative SSH
    6379,   // Redis (normal)
    6667,   // IRC
    8022,   // Alternative SSH (common on routers)
    8080,   // HTTP alt (normal)
    8443,   // HTTPS alt (normal)
    9022,   // Alternative SSH
    53389,  // Alternative RDP
    22222,  // Alternative SSH
    27017,  // MongoDB (normal)
];

/// Ports commonly used by backdoors or trojans
const SUSPICIOUS_PORTS: &[u16] = &[
    12345,  // NetBus default
    12346,  // NetBus
    31337,  // Back Orifice default
    27374,  // SubSeven default
    20034,  // NetBus Pro
    1234,   // UDP binding
    6969,   // Rootkit
];

impl HiddenMapper {
    /// Map hidden and subterranean IPs in a network range
    /// Searches for IPs with non-standard port configurations
    pub async fn map_hidden(range: &str, deep: bool, config: &ScanConfig) -> Vec<ScanResult> {
        println!("[*] Starting hidden IP mapping...");
        println!("[*] Range: {} (deep={})", range, deep);

        // Parse CIDR range or use default
        let subnet = parse_cidr(range).unwrap_or_else(|| {
            network::Subnet {
                network: std::net::Ipv4Addr::new(192, 168, 1, 0),
                mask: 24,
            }
        });

        println!("[*] Scanning subnet: {}/{}", subnet.network, subnet.mask);

        // Get ports to scan based on deep mode
        let ports_to_check = if deep {
            // Deep scan: check both stealth and suspicious ports
            let mut all: Vec<u16> = STEALTH_PORTS.iter().chain(SUSPICIOUS_PORTS.iter()).copied().collect();
            all.sort();
            all.dedup();
            all
        } else {
            // Normal scan: only stealth ports
            STEALTH_PORTS.to_vec()
        };

        println!("[*] Checking {} non-standard ports...", ports_to_check.len());

        // Discover hosts with non-standard ports open
        let hidden_hosts = discover_hidden_hosts(&subnet, &ports_to_check, config).await;

        if hidden_hosts.is_empty() {
            println!("[!] No hidden IPs detected in range.");
            return Vec::new();
        }

        println!("[+] Found {} potential hidden IPs", hidden_hosts.len());

        // Build scan results for each hidden host
        let mut results = Vec::new();
        for host in &hidden_hosts {
            let services: Vec<ServiceInfo> = host.ports.iter()
                .map(|p| ServiceInfo {
                    port: *p,
                    service: get_service_name(*p).to_string(),
                    version: None,
                })
                .collect();

            // Analyze vulnerabilities for hidden services
            let vulnerabilities = analyze_hidden_vulnerabilities(&host.ports);

            let description = classify_hidden_host(&host.ports);
            println!("[+] {} - {}", host.ip, description);

            results.push(ScanResult {
                target: format!("{} ({})", host.ip, description),
                open_ports: host.ports.clone(),
                services,
                vulnerabilities,
                raw_output: format!("Hidden IP discovered: {} with ports {:?}", host.ip, host.ports),
            });
        }

        println!("[*] Hidden IP mapping complete. Found {} hidden services.", results.len());
        results
    }
}

/// Parse CIDR notation string to Subnet
fn parse_cidr(cidr: &str) -> Option<network::Subnet> {
    let parts: Vec<&str> = cidr.split('/').collect();
    if parts.len() != 2 {
        return None;
    }

    let ip: std::net::Ipv4Addr = parts[0].parse().ok()?;
    let mask: u8 = parts[1].parse().ok()?;

    let network = Ipv4Addr::new(ip.octets()[0], ip.octets()[1], ip.octets()[2], 0);

    Some(network::Subnet { network, mask })
}

/// Discover hosts with non-standard ports open
async fn discover_hidden_hosts(subnet: &Subnet, ports: &[u16], config: &ScanConfig) -> Vec<HiddenHost> {
    use tokio::sync::Semaphore;

    let semaphore = Semaphore::new(config.concurrent.min(50));
    let ips = subnet.iter_ips();
    let mut handles = Vec::new();

    for ip in ips {
        let permit = semaphore.acquire().await.unwrap();
        let ports = ports.to_vec();
        let timeout_ms = config.timeout_ms;

        let handle = tokio::spawn(async move {
            let open_ports = scan_stealth_ports(&ip.to_string(), &ports, timeout_ms).await;
            drop(permit);

            if open_ports.len() >= 2 {
                Some(HiddenHost {
                    ip: ip.to_string(),
                    ports: open_ports,
                })
            } else {
                None
            }
        });
        handles.push(handle);
    }

    let mut hosts = Vec::new();
    for handle in handles {
        if let Ok(Some(host)) = handle.await {
            hosts.push(host);
        }
    }

    hosts
}

/// Scan a list of stealth ports on a target
async fn scan_stealth_ports(target: &str, ports: &[u16], timeout_ms: u64) -> Vec<u16> {
    use std::time::Duration;
    use tokio::net::TcpStream;
    use tokio::time::timeout;

    let mut open = Vec::new();

    for port in ports {
        let addr = format!("{}:{}", target, port);
        let timeout_duration = Duration::from_millis(timeout_ms);

        if let Ok(Ok(_)) = timeout(timeout_duration, TcpStream::connect(&addr)).await {
            open.push(*port);
        }
    }

    open
}

/// Simple hidden host info
struct HiddenHost {
    ip: String,
    ports: Vec<u16>,
}

/// Classify what kind of hidden host this might be
fn classify_hidden_host(ports: &[u16]) -> String {
    let has_ssh_alt = ports.contains(&8022) || ports.contains(&9022) || ports.contains(&2222) || ports.contains(&6022);
    let has_telnet_alt = ports.contains(&2323) || ports.contains(&1023);
    let has_rdp_alt = ports.contains(&53389) || ports.contains(&3389);
    let has_suspicious = ports.iter().any(|p| SUSPICIOUS_PORTS.contains(p));

    if has_suspicious {
        "SUSPICIOUS - Possible backdoor ports detected".to_string()
    } else if has_telnet_alt {
        "Stealth Telnet - Non-standard port configuration".to_string()
    } else if has_ssh_alt {
        "Stealth SSH - Alternative SSH port detected".to_string()
    } else if has_rdp_alt {
        "Stealth RDP - Alternative RDP port detected".to_string()
    } else if ports.len() > 3 {
        "Honeypot - Multiple unusual ports open".to_string()
    } else {
        "Hidden Service - Non-standard configuration".to_string()
    }
}

/// Analyze vulnerabilities for hidden services
fn analyze_hidden_vulnerabilities(ports: &[u16]) -> Vec<Vulnerability> {
    let mut vulns = Vec::new();

    for port in ports {
        // Check for SSH on non-standard ports
        if *port == 8022 || *port == 9022 || *port == 2222 || *port == 6022 {
            vulns.push(Vulnerability {
                cve_id: Some("CVE-2023-38408".to_string()),
                description: format!("SSH on non-standard port {} - possible stealth host", port),
                severity: Severity::Info,
                affected_port: *port,
            });
        }

        // Check for Telnet (high risk)
        if *port == 23 || *port == 2323 || *port == 1023 {
            vulns.push(Vulnerability {
                cve_id: None,
                description: format!("Telnet detected on port {} - credentials transmitted in cleartext", port),
                severity: Severity::High,
                affected_port: *port,
            });
        }

        // Check for suspicious ports
        if SUSPICIOUS_PORTS.contains(port) {
            vulns.push(Vulnerability {
                cve_id: None,
                description: format!("Suspicious port {} detected - possible backdoor", port),
                severity: Severity::Critical,
                affected_port: *port,
            });
        }

        // Check for database ports
        if *port == 3306 || *port == 5432 || *port == 27017 || *port == 6379 {
            vulns.push(Vulnerability {
                cve_id: Some("CVE-OPEN".to_string()),
                description: format!("Database port {} exposed on non-standard configuration", port),
                severity: Severity::High,
                affected_port: *port,
            });
        }
    }

    vulns
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_map_hidden_returns_results() {
        let config = ScanConfig {
            target: "192.168.1.0/24".to_string(),
            ports: "8022,9022,2323".to_string(),
            timeout_ms: 1000,
            concurrent: 10,
        };

        let results = HiddenMapper::map_hidden("192.168.1.0/24", false, &config).await;
        // Just verify it doesn't panic
        println!("Hidden map returned {} results", results.len());
    }

    #[test]
    fn test_parse_cidr() {
        let subnet = parse_cidr("192.168.1.0/24").unwrap();
        assert_eq!(subnet.network, std::net::Ipv4Addr::new(192, 168, 1, 0));
        assert_eq!(subnet.mask, 24);
    }

    #[test]
    fn test_classify_hidden_host() {
        let ssh_alt = vec![22, 8022];
        assert!(classify_hidden_host(&ssh_alt).contains("SSH"));

        let suspicious = vec![31337, 12345];
        assert!(classify_hidden_host(&suspicious).contains("SUSPICIOUS"));
    }
}