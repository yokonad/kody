use std::net::SocketAddr;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// Parse port range string into vector of ports
/// Examples: "80", "80,443", "80-443", "80,443,8000-8010"
pub fn parse_port_range(ports: &str) -> Vec<u16> {
    let mut result = Vec::new();
    for part in ports.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if part.contains('-') {
            let parts: Vec<&str> = part.split('-').collect();
            if parts.len() == 2 {
                let start: u16 = parts[0].trim().parse().unwrap_or(1);
                let end: u16 = parts[1].trim().parse().unwrap_or(65535);
                let start = start.min(65535);
                let end = end.min(65535);
                if start <= end {
                    for p in start..=end {
                        result.push(p);
                    }
                }
            }
        } else {
            if let Ok(port) = part.parse::<u16>() {
                if port <= 65535 {
                    result.push(port);
                }
            }
        }
    }
    result.sort();
    result.dedup();
    result
}

/// Attempt TCP connection to target:port within timeout
pub async fn tcp_connect(target: &str, port: u16, timeout_ms: u64) -> bool {
    let addr = format!("{}:{}", target, port);
    let timeout_duration = Duration::from_millis(timeout_ms);

    match timeout(timeout_duration, TcpStream::connect(&addr)).await {
        Ok(Ok(_stream)) => true,
        _ => false,
    }
}

/// TCP scan multiple ports concurrently using semaphore for rate limiting
pub async fn tcp_scan(
    target: &str,
    ports: Vec<u16>,
    concurrency: usize,
    timeout_ms: u64,
) -> Vec<u16> {
    use tokio::sync::Semaphore;

    let semaphore = Semaphore::new(concurrency);
    let mut handles = Vec::new();

    for port in ports {
        let target = target.to_string();
        let permit = semaphore.acquire().await.unwrap();

        let handle = tokio::spawn(async move {
            let result = tcp_connect(&target, port, timeout_ms).await;
            drop(permit);
            if result {
                Some(port)
            } else {
                None
            }
        });
        handles.push(handle);
    }

    let mut open_ports = Vec::new();
    for handle in handles {
        if let Ok(Some(port)) = handle.await {
            open_ports.push(port);
        }
    }

    open_ports.sort();
    open_ports
}

/// Resolve hostname to IP address
pub async fn resolve_host(host: &str) -> Option<String> {
    use std::net::ToSocketAddrs;

    // Check if it's already an IP address
    if host.parse::<std::net::Ipv4Addr>().is_ok() {
        return Some(host.to_string());
    }

    // Try to resolve hostname
    let addr_str = format!("{}:0", host);
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

    #[test]
    fn test_parse_single_port() {
        let ports = parse_port_range("80");
        assert_eq!(ports, vec![80]);
    }

    #[test]
    fn test_parse_multiple_ports() {
        let ports = parse_port_range("80,443");
        assert!(ports.contains(&80));
        assert!(ports.contains(&443));
        assert_eq!(ports.len(), 2);
    }

    #[test]
    fn test_parse_range() {
        let ports = parse_port_range("80-83");
        assert_eq!(ports, vec![80, 81, 82, 83]);
    }

    #[test]
    fn test_parse_mixed() {
        let ports = parse_port_range("22,80-82,443");
        assert!(ports.contains(&22));
        assert!(ports.contains(&80));
        assert!(ports.contains(&81));
        assert!(ports.contains(&82));
        assert!(ports.contains(&443));
        assert_eq!(ports.len(), 5);
    }

    #[test]
    fn test_parse_out_of_range() {
        let ports = parse_port_range("99999");
        assert!(ports.is_empty());
    }

    #[tokio::test]
    async fn test_tcp_connect_localhost() {
        // This test will only pass if something is listening on the port
        // It's more of a demonstration than a real test
        let result = tcp_connect("127.0.0.1", 1, 100).await;
        // Expect false since nothing on port 1
        assert!(!result);
    }
}