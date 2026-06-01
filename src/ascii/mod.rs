/// Simple cross-platform color utilities
pub mod colors {
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

    #[macro_export]
    macro_rules! c {
        ($color:expr, $text:expr) => {
            format!("{}{}{}", $color, $text, RESET)
        };
    }
}

/// Main banner - clean terminal-style with no emojis
pub fn banner() -> String {
    format!(
        r#"
{CYAN}╔══════════════════════════════════════════════════════════════════╗{RESET}
{CYAN}║{RESET}                                                          {CYAN}║{RESET}
{CYAN}║{RESET}                          {BRIGHT_RED}KODY{cyan}                          {CYAN}║{RESET}
{CYAN}║{RESET}                                                          {CYAN}║{RESET}
{CYAN}║{RESET}                      {WHITE}Vulnerability Scanner{white}                  {CYAN}║{RESET}
{CYAN}║{RESET}                          {YELLOW}v0.1.0{white}                        {CYAN}║{RESET}
{CYAN}║{RESET}                          {WHITE}Rust-powered | AI-enabled{white}           {CYAN}║{RESET}
{CYAN}║{RESET}                                                          {CYAN}║{RESET}
{CYAN}╚══════════════════════════════════════════════════════════════════╝{RESET}
"#,
        CYAN = colors::CYAN,
        RESET = colors::RESET,
        BRIGHT_RED = colors::BRIGHT_RED,
        WHITE = colors::WHITE,
        YELLOW = colors::YELLOW
    )
}

/// Scan initiated banner - no emojis
pub fn scan_banner() -> String {
    format!(
        r#"
{CYAN}┌{RESET}{RED}────────────────────────────────────────────────────────────────{CYAN}┐{RESET}
{CYAN}│{RESET}  {RED}SCAN INITIATED{reset}                                                  {CYAN}│{RESET}
{CYAN}└{RESET}{RED}────────────────────────────────────────────────────────────────{CYAN}┘{RESET}
"#,
        CYAN = colors::CYAN,
        RED = colors::RED,
        RESET = colors::RESET
    )
}

/// AI analysis mode banner - no emojis
pub fn ai_banner() -> String {
    format!(
        r#"
{CYAN}┌{RESET}{BRIGHT_CYAN}────────────────────────────────────────────────────────────────{CYAN}┐{RESET}
{CYAN}│{RESET}  {BRIGHT_CYAN}AI ANALYSIS MODE{reset}                                           {CYAN}│{RESET}
{CYAN}└{RESET}{BRIGHT_CYAN}────────────────────────────────────────────────────────────────{CYAN}┘{RESET}
"#,
        CYAN = colors::CYAN,
        BRIGHT_CYAN = colors::BRIGHT_CYAN,
        RESET = colors::RESET
    )
}

/// Auto scan mode banner - no emojis
pub fn auto_scan_banner() -> String {
    format!(
        r#"
{CYAN}┌{RESET}{YELLOW}────────────────────────────────────────────────────────────────{CYAN}┐{RESET}
{CYAN}│{RESET}  {YELLOW}AUTO SCAN MODE{reset}                                               {CYAN}│{RESET}
{CYAN}└{RESET}{YELLOW}────────────────────────────────────────────────────────────────{CYAN}┘{RESET}
"#,
        CYAN = colors::CYAN,
        YELLOW = colors::YELLOW,
        RESET = colors::RESET
    )
}

/// Map hidden banner - no emojis
pub fn map_hidden_banner() -> String {
    format!(
        r#"
{CYAN}┌{RESET}{MAGENTA}────────────────────────────────────────────────────────────────{CYAN}┐{RESET}
{CYAN}│{RESET}  {MAGENTA}MAP HIDDEN MODE{reset}                                             {CYAN}│{RESET}
{CYAN}└{RESET}{MAGENTA}────────────────────────────────────────────────────────────────{CYAN}┘{RESET}
"#,
        CYAN = colors::CYAN,
        MAGENTA = colors::MAGENTA,
        RESET = colors::RESET
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
        // No emoji unicode ranges
        assert!(!b.contains("..."));
        assert!(!b.contains("..."));
        assert!(!b.contains("["));
        // ASCII box drawing only
        assert!(b.contains("╔") || b.contains("┌"));
    }

    #[test]
    fn test_colors_work() {
        assert!(banner().contains("\x1b["));
    }

    #[test]
    fn test_scan_banner_no_emojis() {
        let b = scan_banner();
        assert!(!b.contains("..."));
        assert!(b.contains("SCAN INITIATED"));
    }

    #[test]
    fn test_ai_banner_no_emojis() {
        let b = ai_banner();
        assert!(!b.contains("..."));
        assert!(b.contains("AI ANALYSIS MODE"));
    }
}