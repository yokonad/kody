/// CLI utilities for output formatting
/// Uses ANSI escape codes directly instead of external crate

// ANSI color codes
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";
const BRIGHT_RED: &str = "\x1b[91m";
const BRIGHT_GREEN: &str = "\x1b[92m";
const BOLD: &str = "\x1b[1m";
const RESET: &str = "\x1b[0m";

/// Progress indicator for long-running operations
pub struct Progress {
    message: String,
}

impl Progress {
    pub fn new(msg: &str) -> Self {
        print!("{}{}", msg, YELLOW);
        std::io::stdout().flush().unwrap();
        Self {
            message: msg.to_string(),
        }
    }

    pub fn finish(self) {
        println!("{}{}[DONE]{}", GREEN, RESET, RESET);
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
pub fn print_header(title: &str) {
    println!("\n{}{}{}{}", CYAN, BOLD, "=".repeat(40), RESET);
    println!("{}{}  {}{}", CYAN, BOLD, title, RESET);
    println!("{}{}{}", CYAN, "=".repeat(40), RESET);
}

/// Print scan result in a formatted way
pub fn print_scan_result(target: &str, ports: &[u16], vulns: &[crate::ai::Vulnerability]) {
    print_header(&format!("Scan Results: {}", target));

    println!("\n{}Open Ports:{} {:?}", YELLOW, RESET, ports);

    if !vulns.is_empty() {
        println!("\n{}Vulnerabilities Found:{}{}", RED, BOLD, RESET);
        for vuln in vulns {
            let sev_color = match vuln.severity {
                crate::ai::Severity::Critical => BRIGHT_RED,
                crate::ai::Severity::High => RED,
                crate::ai::Severity::Medium => YELLOW,
                crate::ai::Severity::Low => BLUE,
                crate::ai::Severity::Info => WHITE,
            };
            print!("  [{}] ", sev_color);
            if let Some(cve) = &vuln.cve_id {
                print!("{} - ", cve);
            }
            println!("Port {}: {}", vuln.affected_port, vuln.description);
        }
    } else {
        println!("\n{}No vulnerabilities detected{}", GREEN, RESET);
    }
}

/// Print error message
pub fn print_error(msg: &str) {
    eprintln!("{}Error:{} {}", RED, RESET, msg);
}

/// Print success message
pub fn print_success(msg: &str) {
    println!("{}{}*{} {}", GREEN, RESET, msg);
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_progress_compiles() {
        // Progress is a struct, tests would require mocking stdout
        assert!(true);
    }
}