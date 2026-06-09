//! Terminal aesthetic: GHOST-style banner, boot sequence, and session table.

use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

/// Simple cross-platform color utilities.
pub mod colors {
    #![allow(dead_code)]
    pub const RESET: &str = "\x1b[0m";
    pub const RED: &str = "\x1b[31m";
    pub const GREEN: &str = "\x1b[32m";
    pub const YELLOW: &str = "\x1b[33m";
    pub const BLUE: &str = "\x1b[34m";
    pub const MAGENTA: &str = "\x1b[35m";
    pub const CYAN: &str = "\x1b[36m";
    pub const WHITE: &str = "\x1b[37m";
    pub const GREY: &str = "\x1b[90m";
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BOLD: &str = "\x1b[1m";
    pub const DIM: &str = "\x1b[2m";
}

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Main banner — GHOST-style blocky title (pure ASCII art, colored red).
pub fn banner() -> String {
    let c = colors::BRIGHT_RED;
    let r = colors::RESET;
    let g = colors::GREY;
    format!(
        "\n\
{c} ██╗  ██╗ ██████╗ ██████╗ ██╗   ██╗{r}\n\
{c} ██║ ██╔╝██╔═══██╗██╔══██╗╚██╗ ██╔╝{r}\n\
{c} █████╔╝ ██║   ██║██║  ██║ ╚████╔╝ {r}\n\
{c} ██╔═██╗ ██║   ██║██║  ██║  ╚██╔╝  {r}\n\
{c} ██║  ██╗╚██████╔╝██████╔╝   ██║   {r}\n\
{c} ╚═╝  ╚═╝ ╚═════╝ ╚═════╝    ╚═╝   {r}\n\
{g}      private. dangerous. elite.        KODY v{ver}{r}\n",
        c = c,
        r = r,
        g = g,
        ver = VERSION,
    )
}

/// A pure-ASCII fallback banner (no box-drawing glyphs). Kept for terminals
/// or tests that require ASCII-only output.
pub fn banner_ascii() -> String {
    let c = colors::BRIGHT_RED;
    let r = colors::RESET;
    let g = colors::GREY;
    // Raw string keeps the backslashes literal — the final letter is a real "Y".
    let art = r#"  _  __  ___   ____  __   __
 | |/ / / _ \ |  _ \ \ \ / /
 | ' / | | | || | | | \ V /
 | . \ | |_| || |_| |  | |
 |_|\_\ \___/ |____/   |_|"#;
    format!(
        "\n{c}{art}{r}\n{g}  private. dangerous. elite.   KODY v{ver}{r}\n",
        c = c,
        art = art,
        r = r,
        g = g,
        ver = VERSION,
    )
}

/// Print the boot sequence with a checklist animation.
pub fn boot_sequence() {
    let steps = [
        "establishing secure channel",
        "loading vulnerability signatures",
        "importing CVE database",
        "verifying scan engine",
        "initializing rate limiter",
        "arming session vault",
    ];
    let g = colors::GREY;
    let ok = colors::BRIGHT_GREEN;
    let r = colors::RESET;
    let mut out = std::io::stdout();
    for step in steps {
        print!("{g}[ {step} ]{r}");
        let _ = out.flush();
        sleep(Duration::from_millis(70));
        // pad so the [OK] marks line up roughly
        let pad = 34usize.saturating_sub(step.len());
        println!("{}{ok} ... [✓]{r}", " ".repeat(pad));
    }
}

/// Print the GHOST-style session table with live operational details.
pub fn session_table(operator: &str, egress: &str, ttps_loaded: usize) {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
    let label = colors::GREY;
    let val = colors::WHITE;
    let red = colors::BRIGHT_RED;
    let r = colors::RESET;

    let row = |k: &str, v: String| {
        println!("{label}{k:<14}{r}{val}{v}{r}", k = k, v = v);
    };

    println!();
    row("SESSION START", now.to_string());
    row("TUNNEL", "direct".to_string());
    println!("{label}{:<14}{r}{red}{}{r}", "EGRESS IP", egress, label = label, r = r, red = red);
    println!("{label}{:<14}{r}{red}[REDACTED]{r}", "REGION", label = label, r = r, red = red);
    row("VERSION", format!("kody v{}", VERSION));
    row("OPERATOR", operator.to_string());
    row("TTPs LOADED", ttps_loaded.to_string());
    println!();
}

/// Auto scan mode banner.
pub fn auto_scan_banner() -> String {
    section_banner("AUTO SCAN MODE", colors::YELLOW)
}

/// Map hidden banner.
pub fn map_hidden_banner() -> String {
    section_banner("MAP HIDDEN MODE", colors::MAGENTA)
}

fn section_banner(title: &str, color: &str) -> String {
    let c = colors::CYAN;
    let r = colors::RESET;
    format!(
        "\n{c}+{line}+{r}\n{c}|{r}  {color}{title}{r}{pad}{c}|{r}\n{c}+{line}+{r}\n",
        c = c,
        r = r,
        color = color,
        title = title,
        line = "-".repeat(60),
        pad = " ".repeat(58usize.saturating_sub(title.len())),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_contains_kody() {
        assert!(banner().contains("KODY"));
        assert!(banner_ascii().contains("KODY"));
    }

    #[test]
    fn test_banner_contains_version() {
        let expected = concat!("v", env!("CARGO_PKG_VERSION"));
        assert!(banner().contains(expected), "banner should contain {}", expected);
        assert!(banner_ascii().contains(expected));
    }

    #[test]
    fn test_ascii_banner_is_ascii() {
        // The ASCII fallback must render the same on every terminal.
        assert!(banner_ascii().is_ascii(), "banner_ascii must be pure ASCII");
    }

    #[test]
    fn test_colors_present() {
        assert!(banner().contains("\x1b["));
    }
}
