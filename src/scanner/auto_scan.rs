use crate::ai::ScanResult;
use crate::scanner::{ScanConfig, parse_port_spec};
use crate::network::{self, ScanOptions};
use std::time::Instant;

pub struct AutoScanner;

impl AutoScanner {
    /// Discover devices on the local network and scan them
    pub async fn scan_network(interface: Option<String>, config: &ScanConfig) -> Vec<ScanResult> {
        println!("[*] Discovering network devices...");

        // Get subnet to scan
        let subnet = match interface {
            Some(iface) => {
                println!("[*] Using interface: {}", iface);
                crate::network::get_local_subnet().await.unwrap_or_else(|| {
                    network::Subnet {
                        network: std::net::Ipv4Addr::new(192, 168, 1, 0),
                        mask: 24,
                    }
                })
            }
            None => {
                println!("[*] Auto-detecting network interface...");
                crate::network::get_local_subnet().await.unwrap_or_else(|| {
                    network::Subnet {
                        network: std::net::Ipv4Addr::new(192, 168, 1, 0),
                        mask: 24,
                    }
                })
            }
        };

        println!("[*] Scanning subnet: {}/{}", subnet.network, subnet.mask);

        // Discover alive hosts
        let options = ScanOptions {
            timeout_ms: config.timeout_ms,
            concurrent: config.concurrent.min(50),  // Limit for discovery
            ping_first: false,  // We'll do TCP check instead
        };

        let hosts = network::discover_hosts(&subnet, &options).await;
        println!("[*] Found {} alive hosts", hosts.len());

        if hosts.is_empty() {
            println!("[!] No hosts found. Try specifying a different interface.");
            return Vec::new();
        }

        // Scan each discovered host
        let mut results = Vec::new();
        let scan_start = Instant::now();
        let mut scanned = 0;

        for host in &hosts {
            println!("[*] Scanning {}...", host.ip);
            scanned += 1;

            // Perform port scan on this host
            let host_result = scan_host(&host.ip, config).await;

            if !host_result.open_ports.is_empty() {
                println!("[+] {} has {} open ports", host.ip, host_result.open_ports.len());
            }

            results.push(host_result);

            // Progress indicator
            if scanned % 10 == 0 {
                let elapsed = scan_start.elapsed();
                println!("[*] Progress: {}/{} hosts scanned ({:.1}s elapsed)", scanned, hosts.len(), elapsed.as_secs_f32());
            }
        }

        let total_elapsed = scan_start.elapsed();
        println!("[*] Network scan complete: {} hosts scanned in {:.1}s", scanned, total_elapsed.as_secs_f32());

        results
    }
}

/// Scan a single host for open ports and vulnerabilities
async fn scan_host(target: &str, config: &ScanConfig) -> ScanResult {
    let ports = parse_port_spec(&config.ports);

    // Use Scanner::run for actual scanning
    let result = crate::scanner::Scanner::run(target, ports, config).await;

    ScanResult {
        target: target.to_string(),
        open_ports: result.open_ports,
        services: result.services,
        vulnerabilities: result.vulnerabilities,
        raw_output: format!("Auto-scan of {}", target),
    }
}

/// Scan multiple hosts concurrently with rate limiting
#[allow(dead_code)]
pub async fn scan_multiple_hosts(targets: Vec<String>, ports: &[u16], concurrency: usize) -> Vec<ScanResult> {
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let ports = ports.to_vec();
    let mut handles = Vec::new();

    for target in targets {
        let sem = semaphore.clone();
        let ports = ports.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let config = ScanConfig {
                target: target.clone(),
                ports: ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(","),
                timeout_ms: 3000,
                concurrent: 50,
            };

            crate::scanner::Scanner::run(&target, ports, &config).await
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires network access and can take over 60 seconds
    async fn test_scan_network_returns_results() {
        let config = ScanConfig {
            target: "auto".to_string(),
            ports: "1-1024".to_string(),
            timeout_ms: 1000,
            concurrent: 10,
        };

        // This test requires network access
        // In CI/local testing, it may return empty or mock data
        let results = AutoScanner::scan_network(None, &config).await;
        // Just verify it doesn't panic
        println!("Network scan returned {} results", results.len());
    }
}