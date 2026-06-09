//! HTTP(S) recon: server fingerprint + security-header / hygiene findings.
//!
//! For each web port we issue one request and turn the response headers into
//! actionable findings — what's missing, what an attacker could do with it, and
//! where. This is defensive risk assessment, not exploitation.

use std::time::Duration;
use super::banner::{parse_product_version, Fingerprint};
use super::vuln_rules::{Severity, Vulnerability};

/// Fetch a web port and return its fingerprint plus header-based findings.
pub async fn analyze_web(
    target: &str,
    port: u16,
    tls: bool,
    timeout_ms: u64,
) -> (Fingerprint, Vec<Vulnerability>) {
    let scheme = if tls { "https" } else { "http" };
    let url = format!("{}://{}:{}/", scheme, target, port);
    let mut findings: Vec<Vulnerability> = Vec::new();
    let mut fp = Fingerprint::default();

    let client = match reqwest::Client::builder()
        .timeout(Duration::from_millis(timeout_ms.max(2500)))
        .danger_accept_invalid_certs(true)
        .redirect(reqwest::redirect::Policy::none())
        .build()
    {
        Ok(c) => c,
        Err(_) => return (fp, findings),
    };

    let resp = match client.get(&url).send().await {
        Ok(r) => r,
        Err(_) => return (fp, findings),
    };

    let status = resp.status();
    let headers = resp.headers().clone();

    let header = |name: &str| -> Option<String> {
        headers.get(name).and_then(|v| v.to_str().ok()).map(|s| s.to_string())
    };
    let has = |name: &str| headers.contains_key(name);

    let loc = format!("cabecera HTTP en puerto {}", port);
    let mut push = |desc: &str, sev: Severity, impact: &str| {
        findings.push(
            Vulnerability::new(None, desc, sev, port)
                .with_impact(impact)
                .with_location(&loc),
        );
    };

    // ── Fingerprint + version disclosure ────────────────────────────────────
    if let Some(server) = header("server") {
        let (product, version) = parse_product_version(&server);
        let had_version = version.is_some();
        fp = Fingerprint {
            product,
            version,
            banner: Some(server.clone()),
        };
        if had_version {
            push(
                &format!("El servidor revela su versión en la cabecera Server: {}", server),
                Severity::Info,
                "Fingerprinting: facilita buscar exploits públicos específicos de esa versión.",
            );
        }
    }
    if let Some(xp) = header("x-powered-by") {
        push(
            &format!("Cabecera X-Powered-By revela tecnología: {}", xp),
            Severity::Info,
            "Fingerprinting de la pila tecnológica: orienta al atacante hacia exploits concretos.",
        );
    }

    // ── Security headers ────────────────────────────────────────────────────
    if tls && !has("strict-transport-security") {
        push(
            "Falta HSTS (Strict-Transport-Security).",
            Severity::Medium,
            "Permite SSL-stripping/downgrade: un atacante en la red fuerza HTTP y roba sesión/credenciales.",
        );
    }
    let csp = header("content-security-policy");
    if csp.is_none() {
        push(
            "Falta Content-Security-Policy (CSP).",
            Severity::Low,
            "Facilita XSS: un script inyectado puede robar cookies de sesión o desfigurar la página.",
        );
    }
    let csp_has_ancestors = csp
        .as_deref()
        .map(|c| c.to_lowercase().contains("frame-ancestors"))
        .unwrap_or(false);
    if !has("x-frame-options") && !csp_has_ancestors {
        push(
            "Falta X-Frame-Options / frame-ancestors.",
            Severity::Low,
            "Clickjacking: la página puede embeberse en un iframe para engañar al usuario (clics robados).",
        );
    }
    if !has("x-content-type-options") {
        push(
            "Falta X-Content-Type-Options: nosniff.",
            Severity::Low,
            "MIME sniffing: el navegador puede tratar archivos como otro tipo y ejecutar contenido inesperado.",
        );
    }

    // ── HTTP → HTTPS redirect ───────────────────────────────────────────────
    if !tls {
        let redirects_https = status.is_redirection()
            && header("location").map(|l| l.starts_with("https")).unwrap_or(false);
        if !redirects_https {
            push(
                "HTTP no redirige a HTTPS.",
                Severity::Medium,
                "El tráfico (sesiones, formularios, credenciales) puede quedarse en claro e interceptarse.",
            );
        }
    }

    // ── Cookie hygiene (flag once) ──────────────────────────────────────────
    let mut insecure_cookie = false;
    let mut non_httponly = false;
    for sc in headers.get_all("set-cookie") {
        if let Ok(s) = sc.to_str() {
            let low = s.to_lowercase();
            if tls && !low.contains("secure") {
                insecure_cookie = true;
            }
            if !low.contains("httponly") {
                non_httponly = true;
            }
        }
    }
    if insecure_cookie {
        push(
            "Cookie de sesión sin atributo Secure.",
            Severity::Medium,
            "La cookie puede enviarse por HTTP y ser robada por un atacante en la red (MITM).",
        );
    }
    if non_httponly {
        push(
            "Cookie sin atributo HttpOnly.",
            Severity::Medium,
            "JavaScript (vía XSS) puede leer la cookie de sesión y secuestrar la cuenta.",
        );
    }

    (fp, findings)
}
