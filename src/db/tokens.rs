//! Token management - store and retrieve successful AI tokens

use rusqlite::{Connection, Result as SqliteResult};
use sha2::{Sha256, Digest};

/// A cached AI token with usage statistics
#[derive(Debug, Clone)]
pub struct CachedToken {
    pub id: i64,
    pub provider: String,
    pub token_hash: String,
    pub token_prefix: String,
    pub use_count: i64,
    pub success_rate: f64,
    pub last_used: String,
}

/// Token manager - handles token caching and retrieval
pub struct TokenManager<'a> {
    conn: &'a Connection,
}

impl<'a> TokenManager<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Hash a token for storage (never store raw tokens)
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// Get first 8 characters of token for identification
    pub fn token_prefix(token: &str) -> String {
        token.chars().take(8).collect()
    }

    /// Save a successful token after it worked in an AI call
    pub fn save_successful_token(
        &self,
        provider: &str,
        token: &str,
    ) -> SqliteResult<()> {
        let token_hash = Self::hash_token(token);
        let token_prefix = Self::token_prefix(token);

        // Check if token already exists
        let existing: Option<i64> = self.conn.query_row(
            "SELECT id FROM successful_tokens WHERE token_hash = ?1",
            [&token_hash],
            |row| row.get(0),
        ).ok();

        if let Some(id) = existing {
            // Update existing token
            self.conn.execute(
                "UPDATE successful_tokens SET
                    use_count = use_count + 1,
                    success_count = success_count + 1,
                    success_rate = CAST(success_count AS REAL) / CAST(use_count AS REAL),
                    last_used = CURRENT_TIMESTAMP
                WHERE id = ?1",
                [id],
            )?;
        } else {
            // Insert new token
            self.conn.execute(
                "INSERT INTO successful_tokens (provider, token_hash, token_prefix, use_count, success_count, success_rate)
                VALUES (?1, ?2, ?3, 1, 1, 1.0)",
                [provider, &token_hash, &token_prefix],
            )?;
        }

        Ok(())
    }

    /// Record a failed token usage (decreases success rate)
    pub fn record_failed_token(&self, provider: &str, token: &str) -> SqliteResult<()> {
        let token_hash = Self::hash_token(token);

        let existing: Option<i64> = self.conn.query_row(
            "SELECT id FROM successful_tokens WHERE token_hash = ?1",
            [&token_hash],
            |row| row.get(0),
        ).ok();

        if let Some(id) = existing {
            self.conn.execute(
                "UPDATE successful_tokens SET
                    use_count = use_count + 1,
                    success_rate = CAST(success_count AS REAL) / CAST(use_count AS REAL),
                    last_used = CURRENT_TIMESTAMP
                WHERE id = ?1",
                [id],
            )?;
        }

        Ok(())
    }

    /// Get the best (most successful) token for a provider
    pub fn get_best_token(&self, provider: &str) -> SqliteResult<Option<CachedToken>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, provider, token_hash, token_prefix, use_count, success_rate, last_used
            FROM successful_tokens
            WHERE provider = ?1 AND success_rate >= 0.7
            ORDER BY success_rate DESC, use_count DESC
            LIMIT 1",
        )?;

        let result = stmt.query_row([provider], |row| {
            Ok(CachedToken {
                id: row.get(0)?,
                provider: row.get(1)?,
                token_hash: row.get(2)?,
                token_prefix: row.get(3)?,
                use_count: row.get(4)?,
                success_rate: row.get(5)?,
                last_used: row.get(6)?,
            })
        });

        match result {
            Ok(token) => Ok(Some(token)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Get all tokens for a provider
    pub fn get_all_tokens(&self, provider: &str) -> SqliteResult<Vec<CachedToken>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, provider, token_hash, token_prefix, use_count, success_rate, last_used
            FROM successful_tokens
            WHERE provider = ?1
            ORDER BY success_rate DESC, use_count DESC",
        )?;

        let tokens = stmt.query_map([provider], |row| {
            Ok(CachedToken {
                id: row.get(0)?,
                provider: row.get(1)?,
                token_hash: row.get(2)?,
                token_prefix: row.get(3)?,
                use_count: row.get(4)?,
                success_rate: row.get(5)?,
                last_used: row.get(6)?,
            })
        })?;

        tokens.collect()
    }

    /// Delete a token by ID
    pub fn delete_token(&self, id: i64) -> SqliteResult<()> {
        self.conn.execute("DELETE FROM successful_tokens WHERE id = ?1", [id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn test_db() -> Connection {
        let temp_file = NamedTempFile::new().unwrap();
        let conn = Connection::open(temp_file.path()).unwrap();
        crate::db::schema::create_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn test_hash_token() {
        let hash = TokenManager::hash_token("sk-test123456789");
        assert_eq!(hash.len(), 64); // SHA256 produces 64 hex chars
    }

    #[test]
    fn test_token_prefix() {
        let prefix = TokenManager::token_prefix("sk-test123456789");
        assert_eq!(prefix, "sk-test1");
    }

    #[test]
    fn test_save_and_retrieve_token() {
        let conn = test_db();
        let tm = TokenManager::new(&conn);

        tm.save_successful_token("openai", "sk-testabcdefghij").unwrap();

        let best = tm.get_best_token("openai").unwrap();
        assert!(best.is_some());
        let token = best.unwrap();
        assert_eq!(token.provider, "openai");
        assert_eq!(token.token_prefix, "sk-testab");
    }

    #[test]
    fn test_get_best_token_empty() {
        let conn = test_db();
        let tm = TokenManager::new(&conn);

        let best = tm.get_best_token("openai").unwrap();
        assert!(best.is_none());
    }
}