//! Database schema definitions and migrations

use rusqlite::{Connection, Result as SqliteResult};

/// Create all database tables if they don't exist
pub fn create_tables(conn: &Connection) -> SqliteResult<()> {
    conn.execute_batch(
        r#"
        -- Schema version tracking
        CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY,
            applied_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        -- Successful AI tokens that worked during scans
        CREATE TABLE IF NOT EXISTS successful_tokens (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            provider TEXT NOT NULL,
            token_hash TEXT NOT NULL,
            token_prefix TEXT NOT NULL,
            use_count INTEGER DEFAULT 0,
            success_count INTEGER DEFAULT 0,
            success_rate REAL DEFAULT 0.0,
            last_used TEXT DEFAULT CURRENT_TIMESTAMP,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        -- Create index for fast token lookup
        CREATE INDEX IF NOT EXISTS idx_tokens_provider ON successful_tokens(provider);
        CREATE INDEX IF NOT EXISTS idx_tokens_hash ON successful_tokens(token_hash);

        -- Cached scan methods/techniques that proved effective
        CREATE TABLE IF NOT EXISTS method_cache (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            method_name TEXT NOT NULL,
            target_pattern TEXT,
            port_range TEXT,
            success_score REAL DEFAULT 0.0,
            times_used INTEGER DEFAULT 0,
            last_verified TEXT,
            created_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        -- Create index for method lookup
        CREATE INDEX IF NOT EXISTS idx_methods_name ON method_cache(method_name);

        -- Vulnerability patterns discovered during scans
        CREATE TABLE IF NOT EXISTS vulnerability_cache (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            cve_id TEXT UNIQUE,
            description TEXT,
            severity TEXT,
            affected_ports TEXT,
            verified INTEGER DEFAULT 0,
            discovered_at TEXT DEFAULT CURRENT_TIMESTAMP
        );

        -- Create index for CVE lookup
        CREATE INDEX IF NOT EXISTS idx_vulns_cve ON vulnerability_cache(cve_id);

        -- Scan session history
        CREATE TABLE IF NOT EXISTS scan_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            target TEXT NOT NULL,
            scan_type TEXT NOT NULL,
            ports_scanned TEXT,
            vulnerabilities_found INTEGER DEFAULT 0,
            ai_used INTEGER DEFAULT 0,
            duration_ms INTEGER,
            timestamp TEXT DEFAULT CURRENT_TIMESTAMP
        );

        -- Create index for scan history lookup
        CREATE INDEX IF NOT EXISTS idx_history_target ON scan_history(target);
        CREATE INDEX IF NOT EXISTS idx_history_timestamp ON scan_history(timestamp);

        -- Insert schema version if not exists
        INSERT OR IGNORE INTO schema_version (version) VALUES (1);
        "#,
    )?;

    Ok(())
}

/// Current schema version
#[allow(dead_code)]
pub const CURRENT_VERSION: i32 = 1;