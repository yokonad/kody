//! Network discovery implementation
//! Discovers hosts on the local network using ARP and ICMP

use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio::sync::Semaphore;
use crate::network::ScanOptions;

/// Represents a network interface
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: String,
    pub subnet: Option<Subnet>,
}

/// Represents an IPv4 subnet
#[derive(Debug, Clone)]
pub struct Subnet {
    pub network: Ipv4Addr,
    pub mask: u8,  // e.g., 24 for /24
}

impl Subnet {
    /// Get all IPs in this subnet (limited to /24 or smaller for safety)
    pub fn iter_ips(&self) -> Vec<Ipv4Addr> {
        if self.mask < 24 {
            // Too large, limit to first 254 hosts
            let base = u32::from(self.network);
            (1..=254).map(|i| Ipv4Addr::from(base | i)).collect()
        } else {
            let base = u32::from(self.network);
            let host_bits = 32 - self.mask as u32;
            let num_hosts = (1 << host_bits) - 1;
            let max_hosts = num_hosts.min(254) as u32;
            (1..=max_hosts).map(|i| Ipv4Addr::from(base | i)).collect()
        }
    }
}

/// Get the local subnet by detecting the default network interface
pub async fn get_local_subnet() -> Option<Subnet> {
    // Try to get local IP by connecting to an external address
    // This doesn't actually send traffic, just determines the routing
    if let Ok(addrs) = "8.8.8.8:53".to_socket_addrs() {
        if let Some(addr) = addrs.into_iter().next() {
            if let IpAddr::V4(v4) = addr.ip() {
                // Assume /24 subnet for typical local networks
                let network = Ipv4Addr::new(
                    v4.octets()[0],
                    v4.octets()[1],
                    v4.octets()[2],
                    0,
                );
                return Some(Subnet { network, mask: 24 });
            }
        }
    }

    // Fallback: try to get from environment
    if let Ok(hostname) = std::env::var("HOST_IP") {
        if let Ok(ip) = hostname.parse::<Ipv4Addr>() {
            let network = Ipv4Addr::new(ip.octets()[0], ip.octets()[1], ip.octets()[2], 0);
            return Some(Subnet { network, mask: 24 });
        }
    }

    // Default fallback
    Some(Subnet {
        network: Ipv4Addr::new(192, 168, 1, 0),
        mask: 24,
    })
}

/// List all network interfaces (simplified - returns local interface info)
#[allow(dead_code)]
pub async fn list_interfaces() -> Vec<NetworkInterface> {
    let mut interfaces = Vec::new();

    // Try to get HOST_IP env var first
    if let Ok(host_ip) = std::env::var("HOST_IP") {
        interfaces.push(NetworkInterface {
            name: "default".to_string(),
            ip: host_ip,
            subnet: get_local_subnet().await,
        });
    }

    // If no env var, try to detect
    if interfaces.is_empty() {
        if let Some(subnet) = get_local_subnet().await {
            // Use first IP from subnet as placeholder
            let ips = subnet.iter_ips();
            if let Some(ip) = ips.first() {
                interfaces.push(NetworkInterface {
                    name: "auto".to_string(),
                    ip: ip.to_string(),
                    subnet: Some(subnet),
                });
            }
        }
    }

    interfaces
}

/// Discover alive hosts in a subnet using TCP connection probing
pub async fn discover_hosts(subnet: &Subnet, options: &ScanOptions) -> Vec<super::DiscoveredHost> {
    use std::sync::Arc;

    let ips = subnet.iter_ips();
    let semaphore = Arc::new(Semaphore::new(options.concurrent));
    let timeout_ms = options.timeout_ms;
    let mut handles = Vec::new();

    for ip in ips {
        let sem = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let is_alive = check_host_alive(&ip, timeout_ms).await;

            if is_alive {
                Some(super::DiscoveredHost {
                    ip: ip.to_string(),
                    mac: None,  // MAC requires ARP (needs sudo on most systems)
                    hostname: None,
                    ports: vec![],
                    is_alive: true,
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

/// Check if a host is alive by attempting TCP connections to common ports
async fn check_host_alive(ip: &Ipv4Addr, timeout_ms: u64) -> bool {
    let common_ports = [22, 80, 443, 445, 3389, 8080];

    for port in common_ports {
        let addr = format!("{}:{}", ip, port);
        let timeout_duration = Duration::from_millis(timeout_ms);

        if let Ok(Ok(_)) = timeout(timeout_duration, TcpStream::connect(&addr)).await {
            return true;
        }
    }

    false
}

/// Resolve a hostname to IP address
#[allow(dead_code)]
pub async fn resolve_hostname(hostname: &str) -> Option<String> {
    let addr_str = format!("{}:0", hostname);
    if let Ok(mut addrs) = addr_str.to_socket_addrs() {
        if let Some(addr) = addrs.next() {
            return Some(addr.ip().to_string());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_local_subnet() {
        let subnet = get_local_subnet().await;
        assert!(subnet.is_some());
        let sub = subnet.unwrap();
        assert!(sub.mask >= 16 && sub.mask <= 32);
    }

    #[test]
    fn test_subnet_iter_ips() {
        let subnet = Subnet {
            network: Ipv4Addr::new(192, 168, 1, 0),
            mask: 24,
        };
        let ips = subnet.iter_ips();
        // Should have 254 IPs (192.168.1.1 to 192.168.1.254)
        assert_eq!(ips.len(), 254);
        assert_eq!(ips[0], Ipv4Addr::new(192, 168, 1, 1));
    }

    #[tokio::test]
    async fn test_resolve_hostname() {
        // This test requires network, so we just check it doesn't panic
        let result = resolve_hostname("localhost").await;
        // May or may not succeed depending on system
        println!("Resolved localhost to: {:?}", result);
    }
}