//! Network discovery implementation
//! Discovers hosts on the local network using ARP and ICMP

use std::net::{IpAddr, Ipv4Addr, ToSocketAddrs, UdpSocket};
use std::time::Duration;
use futures::stream::{self, StreamExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
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

/// Maximum number of hosts we will enumerate from a single subnet, to keep
/// large ranges (e.g. /16, /8) from turning into an unbounded scan.
const MAX_HOSTS: u64 = 1024;

impl Subnet {
    /// Compute the canonical network address for this subnet (ip & netmask).
    pub fn network_address(&self) -> Ipv4Addr {
        let mask = self.mask.min(32);
        let host_bits = 32 - mask as u32;
        let netmask = if mask == 0 { 0 } else { u32::MAX << host_bits };
        Ipv4Addr::from(u32::from(self.network) & netmask)
    }

    /// Enumerate usable host IPs in this subnet, honouring the mask and capped
    /// at `MAX_HOSTS` for safety. Excludes the network and broadcast addresses.
    pub fn iter_ips(&self) -> Vec<Ipv4Addr> {
        let mask = self.mask.min(32);
        let host_bits = 32 - mask as u32;
        let net = u32::from(self.network_address());

        if host_bits == 0 {
            return vec![Ipv4Addr::from(net)];
        }
        let total = 1u64 << host_bits; // total addresses in the block
        let last = total.saturating_sub(2).min(MAX_HOSTS); // drop network + broadcast
        (1..=last as u32).map(|i| Ipv4Addr::from(net + i)).collect()
    }
}

/// Detect this machine's primary local IPv4 address.
///
/// Opens a UDP socket and "connects" it to a public address. No packets are
/// actually sent, but the OS picks the outbound interface, so `local_addr`
/// reveals the real LAN IP. This is the standard, dependency-free way to learn
/// the local address without parsing routing tables.
pub fn detect_local_ipv4() -> Option<Ipv4Addr> {
    let sock = UdpSocket::bind("0.0.0.0:0").ok()?;
    sock.connect("8.8.8.8:80").ok()?;
    match sock.local_addr().ok()?.ip() {
        IpAddr::V4(v4) if !v4.is_loopback() && !v4.is_unspecified() => Some(v4),
        _ => None,
    }
}

/// Get the local /24 subnet derived from the real local interface address.
pub async fn get_local_subnet() -> Option<Subnet> {
    // Primary path: ask the OS which interface reaches the internet.
    if let Some(v4) = detect_local_ipv4() {
        let o = v4.octets();
        return Some(Subnet {
            network: Ipv4Addr::new(o[0], o[1], o[2], 0),
            mask: 24,
        });
    }

    // Fallback: explicit override via env var.
    if let Ok(host_ip) = std::env::var("HOST_IP") {
        if let Ok(ip) = host_ip.parse::<Ipv4Addr>() {
            let o = ip.octets();
            return Some(Subnet {
                network: Ipv4Addr::new(o[0], o[1], o[2], 0),
                mask: 24,
            });
        }
    }

    // Last resort: the most common home-network default.
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

/// Discover alive hosts in a subnet using concurrent TCP connection probing.
pub async fn discover_hosts(subnet: &Subnet, options: &ScanOptions) -> Vec<super::DiscoveredHost> {
    let ips = subnet.iter_ips();
    let timeout_ms = options.timeout_ms;
    let concurrency = options.concurrent.max(1);

    stream::iter(ips)
        .map(|ip| async move {
            if check_host_alive(&ip, timeout_ms).await {
                Some(super::DiscoveredHost {
                    ip: ip.to_string(),
                    mac: None, // MAC requires ARP (needs sudo on most systems)
                    hostname: None,
                    ports: vec![],
                    is_alive: true,
                })
            } else {
                None
            }
        })
        .buffer_unordered(concurrency)
        .filter_map(|h| async move { h })
        .collect()
        .await
}

/// Check if a host is alive by racing TCP connections to common ports.
/// Returns as soon as any probe connects, so a live host is detected fast.
async fn check_host_alive(ip: &Ipv4Addr, timeout_ms: u64) -> bool {
    const PROBE_PORTS: &[u16] = &[80, 443, 22, 445, 3389, 8080, 23, 53, 139, 8443];

    stream::iter(PROBE_PORTS)
        .map(|&port| {
            let addr = format!("{}:{}", ip, port);
            async move {
                matches!(
                    timeout(Duration::from_millis(timeout_ms), TcpStream::connect(&addr)).await,
                    Ok(Ok(_))
                )
            }
        })
        .buffer_unordered(PROBE_PORTS.len())
        .any(|alive| async move { alive })
        .await
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