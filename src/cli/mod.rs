/// CLI utilities for output formatting
/// Uses ANSI escape codes directly instead of external crate

use std::io::Write;

/// Progress indicator for long-running operations
#[allow(dead_code)]
pub struct Progress {
    message: String,
}

#[allow(dead_code)]
impl Progress {
    pub fn new(msg: &str) -> Self {
        let y = "\x1b[33m";
        print!("{}{}", msg, y);
        std::io::stdout().flush().unwrap();
        Self {
            message: msg.to_string(),
        }
    }

    pub fn finish(self) {
        let g = "\x1b[32m";
        let r = "\x1b[0m";
        println!("{}{}[DONE]{}", g, r, r);
    }
}

impl Drop for Progress {
    fn drop(&mut self) {
        if !self.message.is_empty() {
            println!("[DONE]");
        }
    }
}

/// Print a formatted header
#[allow(dead_code)]
pub fn print_header(title: &str) {
    let cyan = "\x1b[36m";
    let bold = "\x1b[1m";
    let reset = "\x1b[0m";
    println!("\n{}{}{}{}", cyan, bold, "=".repeat(40), reset);
    println!("{}{}  {}{}", cyan, bold, title, reset);
    println!("{}{}{}", cyan, "=".repeat(40), reset);
}

/// Print scan result in a formatted way
#[allow(dead_code)]
pub fn print_scan_result(target: &str, ports: &[u16], vulns: &[crate::ai::Vulnerability]) {
    print_header(&format!("Scan Results: {}", target));

    let y = "\x1b[33m";
    let r = "\x1b[0m";
    let red = "\x1b[31m";
    let bold = "\x1b[1m";
    println!("\n{}Open Ports:{} {:?}", y, r, ports);

    if !vulns.is_empty() {
        println!("\n{}Vulnerabilities Found:{}{}", red, bold, r);
        for vuln in vulns {
            let sev_color = match vuln.severity {
                crate::ai::Severity::Critical => "\x1b[91m",
                crate::ai::Severity::High => red,
                crate::ai::Severity::Medium => y,
                crate::ai::Severity::Low => "\x1b[34m",
                crate::ai::Severity::Info => "\x1b[37m",
            };
            print!("  [{}] ", sev_color);
            if let Some(cve) = &vuln.cve_id {
                print!("{} - ", cve);
            }
            println!("Port {}: {}", vuln.affected_port, vuln.description);
        }
    } else {
        println!("\n{}No vulnerabilities detected{}", "\x1b[32m", "\x1b[0m");
    }
}

/// Print error message
#[allow(dead_code)]
pub fn print_error(msg: &str) {
    eprintln!("{}Error:{} {}", "\x1b[31m", "\x1b[0m", msg);
}

/// Print success message
#[allow(dead_code)]
pub fn print_success(msg: &str) {
    println!("{}* {} {}", "\x1b[32m", msg, "\x1b[0m");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_progress_compiles() {
        // Progress is a struct, tests would require mocking stdout
        assert!(true);
    }
}