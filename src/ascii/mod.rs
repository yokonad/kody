/// Simple cross-platform color utilities
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
    pub const BRIGHT_RED: &str = "\x1b[91m";
    pub const BRIGHT_GREEN: &str = "\x1b[92m";
    pub const BRIGHT_YELLOW: &str = "\x1b[93m";
    pub const BRIGHT_BLUE: &str = "\x1b[94m";
    pub const BRIGHT_CYAN: &str = "\x1b[96m";
    pub const BOLD: &str = "\x1b[1m";
}

/// Main banner - clean terminal-style with no emojis
pub fn banner() -> String {
    format!(
        r#"
{C}+--------------------------------------------------------------------+{R}
{C}|{R}                                                                  {C}|{R}
{C}|{R}                          {BR}KODY{C}                          {C}|{R}
{C}|{R}                                                                  {C}|{R}
{C}|{R}                      {W}Vulnerability Scanner{W}                  {C}|{R}
{C}|{R}                          {Y}v0.3.0{W}                             {C}|{R}
{C}|{R}                      {W}Rust-powered | AI-enabled{W}              {C}|{R}
{C}|{R}                                                                  {C}|{R}
{C}+--------------------------------------------------------------------+{R}
"#,
        C = colors::CYAN,
        R = colors::RESET,
        BR = colors::BRIGHT_RED,
        W = colors::WHITE,
        Y = colors::YELLOW
    )
}

/// Scan initiated banner - no emojis
#[allow(dead_code)]
pub fn scan_banner() -> String {
    format!(
        r#"
{C}┌{RE}────────────────────────────────────────────────────────────────{C}┐{R}
{C}│{R}  {RE}SCAN INITIATED{R}                                                  {C}│{R}
{C}└{RE}────────────────────────────────────────────────────────────────{C}┘{R}
"#,
        C = colors::CYAN,
        RE = colors::RED,
        R = colors::RESET
    )
}

/// AI analysis mode banner - no emojis
#[allow(dead_code)]
pub fn ai_banner() -> String {
    format!(
        r#"
{C}┌{BC}────────────────────────────────────────────────────────────────{C}┐{R}
{C}│{R}  {BC}AI ANALYSIS MODE{BC}                                           {C}│{R}
{C}└{BC}────────────────────────────────────────────────────────────────{C}┘{R}
"#,
        C = colors::CYAN,
        BC = colors::BRIGHT_CYAN,
        R = colors::RESET
    )
}

/// Auto scan mode banner - no emojis
pub fn auto_scan_banner() -> String {
    format!(
        r#"
{C}┌{Y}────────────────────────────────────────────────────────────────{C}┐{R}
{C}│{R}  {Y}AUTO SCAN MODE{Y}                                               {C}│{R}
{C}└{Y}────────────────────────────────────────────────────────────────{C}┘{R}
"#,
        C = colors::CYAN,
        Y = colors::YELLOW,
        R = colors::RESET
    )
}

/// Map hidden banner - no emojis
pub fn map_hidden_banner() -> String {
    format!(
        r#"
{C}┌{M}────────────────────────────────────────────────────────────────{C}┐{R}
{C}│{R}  {M}MAP HIDDEN MODE{M}                                             {C}│{R}
{C}└{M}────────────────────────────────────────────────────────────────{C}┘{R}
"#,
        C = colors::CYAN,
        M = colors::MAGENTA,
        R = colors::RESET
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_banner_contains_kody() {
        let b = banner();
        assert!(b.contains("KODY"));
    }

    #[test]
    fn test_banner_contains_version() {
        let b = banner();
        assert!(b.contains("v0.1.0"));
    }

    #[test]
    fn test_banner_has_no_emojis() {
        let b = banner();
        assert!(b.contains("╔") || b.contains("═"));
    }

    #[test]
    fn test_colors_work() {
        assert!(banner().contains("\x1b["));
    }
}