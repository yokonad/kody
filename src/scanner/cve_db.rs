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
    /// What an attacker could achieve — "qué podría dañar".
    impact: &'static str,
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
        impact: "RCE pre-autenticación como root: control total del servidor (robo/borrado de datos, pivote a la red interna, persistencia).",
    },
    CveRule {
        product: "openssh",
        introduced: None,
        fixed: Some("9.3p2"),
        cve: "CVE-2023-38408",
        severity: Severity::High,
        description: "RCE via forwarded ssh-agent PKCS#11 provider (fixed in 9.3p2)",
        impact: "Ejecución de código si una víctima reenvía su agente SSH (ForwardAgent) contra este host.",
    },
    CveRule {
        product: "apache",
        introduced: Some("2.4.49"),
        fixed: Some("2.4.51"),
        cve: "CVE-2021-41773",
        severity: Severity::Critical,
        description: "Apache httpd path traversal / RCE (2.4.49-2.4.50)",
        impact: "Lectura de archivos arbitrarios (/etc/passwd, código fuente, claves) y RCE si mod_cgi está activo.",
    },
    CveRule {
        product: "nginx",
        introduced: Some("0.6.18"),
        fixed: Some("1.21.0"),
        cve: "CVE-2021-23017",
        severity: Severity::High,
        description: "nginx DNS resolver off-by-one heap write (before 1.21.0)",
        impact: "Corrupción de memoria en el resolver: posible RCE o caída del servicio (DoS).",
    },
    CveRule {
        product: "vsftpd",
        introduced: Some("2.3.4"),
        fixed: Some("2.3.5"),
        cve: "CVE-2011-2523",
        severity: Severity::Critical,
        description: "vsftpd 2.3.4 contains a backdoor giving a root shell",
        impact: "Backdoor: shell root inmediato sin credenciales = control total del servidor.",
    },
    CveRule {
        product: "proftpd",
        introduced: None,
        fixed: Some("1.3.6"),
        cve: "CVE-2019-12815",
        severity: Severity::Critical,
        description: "ProFTPD mod_copy arbitrary file copy / RCE (before 1.3.6)",
        impact: "Copia/lectura/escritura arbitraria de archivos: subir una webshell o robar datos → RCE.",
    },
    CveRule {
        product: "exim",
        introduced: Some("4.87"),
        fixed: Some("4.92"),
        cve: "CVE-2019-10149",
        severity: Severity::Critical,
        description: "Exim 'The Return of the WIZard' RCE (4.87-4.91)",
        impact: "RCE como root vía un email manipulado: control del servidor de correo e intercepción de correo.",
    },
    CveRule {
        product: "samba",
        introduced: Some("3.5.0"),
        fixed: Some("4.6.4"),
        cve: "CVE-2017-7494",
        severity: Severity::Critical,
        description: "SambaCry: remote code execution via writable share (3.5.0-4.6.3)",
        impact: "RCE subiendo una librería a un share escribible: control del servidor de archivos.",
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
            out.push(
                Vulnerability::new(Some(rule.cve), rule.description, rule.severity, port)
                    .with_impact(rule.impact)
                    .with_location(&format!("{} {} en puerto {}", product, version, port)),
            );
        }
    }

    out
}

/// Exposure / best-practice findings for sensitive services. NEVER carries a
/// CVE id — these describe risk, not a confirmed vulnerability.
pub fn exposure_findings(port: u16) -> Vec<Vulnerability> {
    let mut v = Vec::new();
    let mut add = |desc: &str, sev: Severity, impact: &str| {
        v.push(
            Vulnerability::new(None, desc, sev, port)
                .with_impact(impact)
                .with_location(&format!("puerto {}", port)),
        );
    };

    match port {
        23 | 2323 => add(
            "Telnet expuesto: credenciales y datos viajan en texto plano. Usa SSH.",
            Severity::High,
            "Cualquiera en la red captura usuario/contraseña y secuestra la sesión.",
        ),
        21 => add(
            "FTP expuesto: canal de control en claro y posible login anónimo. Prefiere SFTP/FTPS.",
            Severity::Medium,
            "Robo de credenciales y de archivos; con login anónimo, acceso directo a ficheros.",
        ),
        6379 => add(
            "Redis expuesto a la red: por defecto SIN autenticación. Bind a localhost o pon contraseña.",
            Severity::Critical,
            "Lectura/escritura de toda la base y, a menudo, RCE escribiendo claves SSH o cronjobs.",
        ),
        27017 | 27018 => add(
            "MongoDB expuesto a la red: verifica que la autenticación esté activa y con firewall.",
            Severity::Critical,
            "Si está sin auth: volcado o borrado de TODA la base de datos.",
        ),
        11211 => add(
            "Memcached expuesto: sin autenticación por diseño y usable para amplificación DDoS.",
            Severity::Critical,
            "Lectura de datos cacheados y abuso para lanzar DDoS contra terceros desde tu IP.",
        ),
        9200 | 9300 => add(
            "Elasticsearch expuesto: verifica que la seguridad (X-Pack) esté activa.",
            Severity::High,
            "Clúster abierto = volcado de todos los índices (PII, logs, datos sensibles).",
        ),
        3306 => add(
            "MySQL/MariaDB accesible desde la red: restringe a hosts confiables con firewall.",
            Severity::High,
            "Superficie de fuerza bruta; si caen las credenciales, acceso total a la base.",
        ),
        5432 => add(
            "PostgreSQL accesible desde la red: restringe acceso y exige autenticación fuerte.",
            Severity::High,
            "Fuerza bruta de credenciales → lectura/escritura de toda la base.",
        ),
        1433 => add(
            "Microsoft SQL Server accesible desde la red: restringe el acceso con firewall.",
            Severity::High,
            "Fuerza bruta; con xp_cmdshell habilitado puede derivar en RCE en el host.",
        ),
        5984 => add(
            "CouchDB expuesto: verifica credenciales de admin (evita el modo 'admin party').",
            Severity::High,
            "En 'admin party' = control total de la base sin autenticación.",
        ),
        3389 => add(
            "RDP expuesto a la red: principal puerta de entrada de ransomware. Ponlo tras VPN + NLA.",
            Severity::High,
            "Fuerza bruta/credential stuffing → escritorio remoto y movimiento lateral.",
        ),
        5900 | 5901 => add(
            "VNC expuesto: asegúrate de tener contraseña fuerte; muchos permiten auth débil o nula.",
            Severity::High,
            "Control del escritorio remoto: ver pantalla, teclear, instalar malware.",
        ),
        445 | 139 => add(
            "SMB/NetBIOS expuesto: limítalo a redes confiables; vector frecuente de lateral movement.",
            Severity::Medium,
            "Enumeración de shares y movimiento lateral/ransomware (p. ej. EternalBlue en versiones viejas).",
        ),
        80 | 8080 | 8000 | 8008 | 8888 => add(
            "HTTP en claro (sin TLS): el tráfico va sin cifrar. Redirige a HTTPS.",
            Severity::Low,
            "Interceptable en la red: sesiones, formularios y credenciales en texto plano.",
        ),
        25 => add(
            "SMTP expuesto: verifica que no sea open relay y que STARTTLS esté forzado.",
            Severity::Info,
            "Si es open relay: abuso para enviar spam/phishing en tu nombre.",
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
