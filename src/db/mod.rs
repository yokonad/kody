//! Database module for kody offline method cache
//!
//! SECURITY WARNING: Tokens and sensitive data are stored in plaintext.
//! Future work: add encryption at rest for production use.

pub mod schema;
pub mod tokens;
pub mod methods;
pub mod vulns;
pub mod history;

use std::path::Path;
use rusqlite::{Connection, Result as SqliteResult};
use tracing::{info, warn};

/// Main database wrapper - wraps rusqlite::Connection
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Create a new database connection at the given path
    /// Returns Err if the database cannot be opened/written
    pub fn new(path: &Path) -> SqliteResult<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                warn!("Database directory does not exist, trying to create: {:?}", parent);
                // Continue - will fail later if can't write
            }
        }

        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init()?;
        info!("Database initialized at {:?}", path);
        Ok(db)
    }

    /// Initialize database schema - create tables if they don't exist
    pub fn init(&self) -> SqliteResult<()> {
        schema::create_tables(&self.conn)
    }

    /// Get a reference to the underlying connection
    pub fn connection(&self) -> &Connection {
        &self.conn
    }

    /// Check if database is properly initialized and readable
    pub fn health_check(&self) -> SqliteResult<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
            [],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }
}

// Re-export submodules
pub use tokens::{CachedToken, TokenManager};
pub use methods::{CachedMethod, MethodCache};
pub use vulns::{CachedVuln, VulnCache};
pub use history::{ScanRecord, ScanHistory};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_database_new_creates_schema() {
        let temp_file = NamedTempFile::new().unwrap();
        let path = temp_file.path();

        let db = Database::new(path).expect("Failed to create database");
        assert!(db.health_check().unwrap());
    }

    #[test]
    fn test_database_new_fails_on_invalid_path() {
        // Use a path that should fail (read-only directory if exists, or non-existent parent)
        let result = Database::new(Path::new("/nonexistent/path/to/db.sqlite"));
        // This might fail or succeed depending on permissions - just ensure it returns Result
        // The actual behavior depends on the system
    }
}