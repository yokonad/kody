use std::net::ToSocketAddrs;
use std::time::Duration;
use futures::stream::{self, StreamExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

/// ~120 most common TCP ports (nmap top-ports style) plus security-relevant
/// services (databases, caches, admin panels). Used as the fast default.
pub const TOP_PORTS: &[u16] = &[
    7, 20, 21, 22, 23, 25, 26, 37, 53, 79, 80, 81, 88, 106, 110, 111, 113, 119,
    135, 139, 143, 144, 179, 199, 389, 427, 443, 444, 445, 465, 513, 514, 515,
    543, 544, 548, 554, 587, 631, 636, 646, 873, 990, 993, 995, 1025, 1080,
    1433, 1521, 1723, 1900, 2000, 2049, 2121, 2375, 2376, 3000, 3128, 3306,
    3389, 4444, 4899, 5000, 5060, 5432, 5601, 5666, 5800, 5900, 5984, 6000,
    6379, 6443, 6667, 7001, 7070, 8000, 8008, 8009, 8080, 8081, 8086, 8088,
    8443, 8888, 9000, 9090, 9100, 9200, 9300, 9999, 10000, 11211, 15672, 27017,
    27018, 32768, 49152, 49153, 49154,
];

/// Parse a port specification into a list of ports.
///
/// Accepts:
/// - `"top"`  -> the fast default set of common ports
/// - `"full"` -> 1-65535
/// - ranges and lists: `"80"`, `"80,443"`, `"1-1024"`, `"22,80-90,443"`
pub fn parse_port_spec(spec: &str) -> Vec<u16> {
    match spec.trim().to_ascii_lowercase().as_str() {
        "top" | "" | "default" => TOP_PORTS.to_vec(),
        "full" | "all" => (1..=65535).collect(),
        _ => parse_port_range(spec),
    }
}

/// Parse a port range string into a sorted, deduped vector of ports.
/// Examples: "80", "80,443", "80-443", "80,443,8000-8010"
pub fn parse_port_range(ports: &str) -> Vec<u16> {
    let mut result = Vec::new();
    for part in ports.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((a, b)) = part.split_once('-') {
            // Range: skip silently if either bound is not a valid u16.
            if let (Ok(start), Ok(end)) = (a.trim().parse::<u16>(), b.trim().parse::<u16>()) {
                if start <= end {
                    result.extend(start..=end);
                }
            }
        } else if let Ok(port) = part.parse::<u16>() {
            result.push(port);
        }
    }
    result.sort_unstable();
    result.dedup();
    result
}

/// Attempt a TCP connection to target:port within timeout. Returns true if open.
pub async fn tcp_connect(target: &str, port: u16, timeout_ms: u64) -> bool {
    let addr = format!("{}:{}", target, port);
    matches!(
        timeout(Duration::from_millis(timeout_ms), TcpStream::connect(&addr)).await,
        Ok(Ok(_))
    )
}

/// TCP-scan multiple ports concurrently, bounded by `concurrency`.
///
/// Uses a bounded stream instead of spawning one task per port, so scanning the
/// full 65535 range stays cheap and responsive.
pub async fn tcp_scan(
    target: &str,
    ports: Vec<u16>,
    concurrency: usize,
    timeout_ms: u64,
) -> Vec<u16> {
    let concurrency = concurrency.max(1);
    let mut open_ports: Vec<u16> = stream::iter(ports)
        .map(|port| async move {
            if tcp_connect(target, port, timeout_ms).await {
                Some(port)
            } else {
                None
            }
        })
        .buffer_unordered(concurrency)
        .filter_map(|r| async move { r })
        .collect()
        .await;

    open_ports.sort_unstable();
    open_ports
}

/// Resolve a hostname to ALL of its IP addresses (deduped, sorted).
/// A literal IP passes through as a single-element list.
pub async fn resolve_all(host: &str) -> Vec<String> {
    if host.parse::<std::net::IpAddr>().is_ok() {
        return vec![host.to_string()];
    }
    let addr_str = format!("{}:0", host);
    tokio::task::spawn_blocking(move || {
        match addr_str.to_socket_addrs() {
            Ok(addrs) => {
                let mut ips: Vec<String> = addrs.map(|a| a.ip().to_string()).collect();
                ips.sort();
                ips.dedup();
                ips
            }
            Err(_) => Vec::new(),
        }
    })
    .await
    .unwrap_or_default()
}

/// Return true if the target is a bare IP literal (not a hostname/domain).
pub fn is_ip_literal(target: &str) -> bool {
    target.parse::<std::net::IpAddr>().is_ok()
}

/// Resolve a hostname to an IP address. Passes through literal IPs unchanged.
pub async fn resolve_host(host: &str) -> Option<String> {
    if host.parse::<std::net::IpAddr>().is_ok() {
        return Some(host.to_string());
    }

    let addr_str = format!("{}:0", host);
    // to_socket_addrs blocks on DNS; run it off the async runtime.
    let host_owned = addr_str;
    tokio::task::spawn_blocking(move || {
        host_owned
            .to_socket_addrs()
            .ok()
            .and_then(|mut addrs| addrs.next())
            .map(|addr| addr.ip().to_string())
    })
    .await
    .ok()
    .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_single_port() {
        assert_eq!(parse_port_range("80"), vec![80]);
    }

    #[test]
    fn test_parse_multiple_ports() {
        let ports = parse_port_range("80,443");
        assert_eq!(ports, vec![80, 443]);
    }

    #[test]
    fn test_parse_range() {
        assert_eq!(parse_port_range("80-83"), vec![80, 81, 82, 83]);
    }

    #[test]
    fn test_parse_mixed() {
        let ports = parse_port_range("22,80-82,443");
        assert_eq!(ports, vec![22, 80, 81, 82, 443]);
    }

    #[test]
    fn test_parse_out_of_range() {
        // 99999 overflows u16 -> ignored, not clamped to a wrong port.
        assert!(parse_port_range("99999").is_empty());
    }

    #[test]
    fn test_parse_spec_top() {
        assert_eq!(parse_port_spec("top"), TOP_PORTS.to_vec());
        assert_eq!(parse_port_spec(""), TOP_PORTS.to_vec());
    }

    #[test]
    fn test_parse_spec_full() {
        assert_eq!(parse_port_spec("full").len(), 65535);
    }

    #[test]
    fn test_top_ports_sorted_unique() {
        let mut sorted = TOP_PORTS.to_vec();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), TOP_PORTS.len(), "TOP_PORTS must be unique");
    }

    #[tokio::test]
    async fn test_tcp_connect_closed_port() {
        // Nothing should be listening on port 1 of localhost.
        assert!(!tcp_connect("127.0.0.1", 1, 100).await);
    }
}
