//! Banner grabbing and version fingerprinting.
//!
//! For each open port we try to learn the *real* product and version running,
//! so vulnerability matching is based on observed evidence instead of guesses.
//! Two strategies:
//!   - HTTP(S) ports: issue a request and read the `Server` header.
//!   - Everything else: read the greeting many services send on connect
//!     (SSH, FTP, SMTP, POP3, IMAP, ...).

use std::sync::OnceLock;
use std::time::Duration;
use regex::Regex;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::time::timeout;

/// What we managed to fingerprint on a port.
#[derive(Debug, Clone, Default)]
pub struct Fingerprint {
    /// Detected product, e.g. "OpenSSH", "nginx", "Apache".
    pub product: Option<String>,
    /// Detected version, e.g. "9.7p1", "1.18.0", "2.4.52".
    pub version: Option<String>,
    /// Raw banner / Server header, for transparency.
    #[allow(dead_code)]
    pub banner: Option<String>,
}

impl Fingerprint {
    /// A human-readable "product version" string, if anything was detected.
    pub fn display(&self) -> Option<String> {
        match (&self.product, &self.version) {
            (Some(p), Some(v)) => Some(format!("{} {}", p, v)),
            (Some(p), None) => Some(p.clone()),
            (None, Some(v)) => Some(v.clone()),
            (None, None) => None,
        }
    }
}

const HTTP_PORTS: &[u16] = &[
    80, 81, 591, 2080, 3000, 5000, 8000, 8008, 8080, 8081, 8086, 8088, 8888, 9000, 9090,
];
const HTTPS_PORTS: &[u16] = &[443, 8443, 9443, 6443, 4443];

/// `Name/1.2.3` or `Name_1.2.3p1` style tokens (Server headers, SSH banners).
fn product_version_re() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| {
        Regex::new(r"([A-Za-z][A-Za-z0-9.+\-]*)[/_]v?(\d+(?:\.\d+)+(?:p\d+)?)").unwrap()
    })
}

/// Parse a product + version out of an arbitrary banner / header string.
fn parse_product_version(text: &str) -> (Option<String>, Option<String>) {
    let text = text.trim();

    // SSH greeting: "SSH-2.0-OpenSSH_8.9p1 Ubuntu-3ubuntu0.1"
    if let Some(rest) = text.strip_prefix("SSH-2.0-").or_else(|| text.strip_prefix("SSH-1.99-")) {
        let token = rest.split_whitespace().next().unwrap_or(rest);
        if let Some((prod, ver)) = token.split_once('_') {
            return (Some(prod.to_string()), Some(ver.to_string()));
        }
        return (Some(token.to_string()), None);
    }

    if let Some(caps) = product_version_re().captures(text) {
        let product = caps.get(1).map(|m| m.as_str().to_string());
        let version = caps.get(2).map(|m| m.as_str().to_string());
        return (product, version);
    }

    (None, None)
}

/// Fingerprint a single open port.
pub async fn grab_fingerprint(target: &str, port: u16, timeout_ms: u64) -> Fingerprint {
    if HTTP_PORTS.contains(&port) {
        if let Some(fp) = http_fingerprint(target, port, false, timeout_ms).await {
            return fp;
        }
    } else if HTTPS_PORTS.contains(&port) {
        if let Some(fp) = http_fingerprint(target, port, true, timeout_ms).await {
            return fp;
        }
    }

    // Fall back to (or default to) raw banner reading.
    raw_banner(target, port, timeout_ms).await
}

/// Read whatever a service emits on connect.
async fn raw_banner(target: &str, port: u16, timeout_ms: u64) -> Fingerprint {
    let addr = format!("{}:{}", target, port);
    let connect = timeout(Duration::from_millis(timeout_ms), TcpStream::connect(&addr)).await;

    let mut stream = match connect {
        Ok(Ok(s)) => s,
        _ => return Fingerprint::default(),
    };

    let mut buf = vec![0u8; 512];
    // Many services greet immediately; give them a short window.
    let read = timeout(Duration::from_millis(1500), stream.read(&mut buf)).await;

    if let Ok(Ok(n)) = read {
        if n > 0 {
            let banner = String::from_utf8_lossy(&buf[..n]).trim().to_string();
            if !banner.is_empty() {
                let (product, version) = parse_product_version(&banner);
                return Fingerprint {
                    product,
                    version,
                    banner: Some(banner.chars().take(200).collect()),
                };
            }
        }
    }

    Fingerprint::default()
}

/// Fingerprint an HTTP(S) service via its `Server` response header.
async fn http_fingerprint(target: &str, port: u16, tls: bool, timeout_ms: u64) -> Option<Fingerprint> {
    let scheme = if tls { "https" } else { "http" };
    let url = format!("{}://{}:{}/", scheme, target, port);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms.max(2000)))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .ok()?;

    let resp = client.get(&url).send().await.ok()?;

    let server = resp
        .headers()
        .get(reqwest::header::SERVER)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let server = server?;
    let (product, version) = parse_product_version(&server);
    Some(Fingerprint {
        product,
        version,
        banner: Some(server),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssh() {
        let (p, v) = parse_product_version("SSH-2.0-OpenSSH_9.7p1 Ubuntu-3ubuntu0.1");
        assert_eq!(p.as_deref(), Some("OpenSSH"));
        assert_eq!(v.as_deref(), Some("9.7p1"));
    }

    #[test]
    fn test_parse_apache() {
        let (p, v) = parse_product_version("Apache/2.4.52 (Ubuntu)");
        assert_eq!(p.as_deref(), Some("Apache"));
        assert_eq!(v.as_deref(), Some("2.4.52"));
    }

    #[test]
    fn test_parse_nginx() {
        let (p, v) = parse_product_version("nginx/1.18.0");
        assert_eq!(p.as_deref(), Some("nginx"));
        assert_eq!(v.as_deref(), Some("1.18.0"));
    }

    #[test]
    fn test_parse_iis() {
        let (p, v) = parse_product_version("Microsoft-IIS/10.0");
        assert_eq!(p.as_deref(), Some("Microsoft-IIS"));
        assert_eq!(v.as_deref(), Some("10.0"));
    }

    #[test]
    fn test_parse_none() {
        let (p, v) = parse_product_version("cloudflare");
        assert!(p.is_none() && v.is_none());
    }

    #[test]
    fn test_display() {
        let fp = Fingerprint {
            product: Some("nginx".into()),
            version: Some("1.18.0".into()),
            banner: None,
        };
        assert_eq!(fp.display().as_deref(), Some("nginx 1.18.0"));
    }
}
