//! Method cache - store and retrieve successful scan methods/techniques

use rusqlite::{Connection, Result as SqliteResult};

/// A cached scan method with success score
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CachedMethod {
    pub id: i64,
    pub method_name: String,
    pub target_pattern: Option<String>,
    pub port_range: Option<String>,
    pub success_score: f64,
    pub times_used: i64,
    pub last_verified: Option<String>,
    pub created_at: String,
}

/// Method cache manager
#[allow(dead_code)]
pub struct MethodCache<'a> {
    conn: &'a Connection,
}

#[allow(dead_code)]
impl<'a> MethodCache<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Cache a successful scan method
    #[allow(dead_code)]
    pub fn cache_method(
        &self,
        name: &str,
        target_pattern: Option<&str>,
        port_range: Option<&str>,
    ) -> SqliteResult<()> {
        // Check if method already exists
        let existing: Option<i64> = self.conn.query_row(
            "SELECT id FROM method_cache WHERE method_name = ?1",
            rusqlite::params![name],
            |row| row.get(0),
        ).ok();

        if let Some(id) = existing {
            // Update existing method
            self.conn.execute(
                "UPDATE method_cache SET
                    success_score = success_score + 0.1,
                    times_used = times_used + 1,
                    last_verified = CURRENT_TIMESTAMP
                WHERE id = ?1",
                rusqlite::params![id],
            )?;
        } else {
            // Insert new method
            self.conn.execute(
                "INSERT INTO method_cache (method_name, target_pattern, port_range, success_score, times_used)
                VALUES (?1, ?2, ?3, 0.8, 1)",
                rusqlite::params![name, target_pattern.unwrap_or(""), port_range.unwrap_or("")],
            )?;
        }

        Ok(())
    }

    /// Get cached methods that match a target pattern
    pub fn get_cached_methods(&self, target_pattern: Option<&str>) -> SqliteResult<Vec<CachedMethod>> {
        let mut methods = Vec::new();

        if let Some(pattern) = target_pattern {
            let mut stmt = self.conn.prepare(
                "SELECT id, method_name, target_pattern, port_range, success_score, times_used, last_verified, created_at
                FROM method_cache
                WHERE target_pattern LIKE ?1 OR target_pattern IS NULL
                ORDER BY success_score DESC, times_used DESC
                LIMIT 20",
            )?;

            let rows = stmt.query_map([pattern], |row| {
                Ok(CachedMethod {
                    id: row.get(0)?,
                    method_name: row.get(1)?,
                    target_pattern: row.get(2)?,
                    port_range: row.get(3)?,
                    success_score: row.get(4)?,
                    times_used: row.get(5)?,
                    last_verified: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?;

            for method in rows {
                methods.push(method?);
            }
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, method_name, target_pattern, port_range, success_score, times_used, last_verified, created_at
                FROM method_cache
                ORDER BY success_score DESC, times_used DESC
                LIMIT 20",
            )?;

            let rows = stmt.query_map([], |row| {
                Ok(CachedMethod {
                    id: row.get(0)?,
                    method_name: row.get(1)?,
                    target_pattern: row.get(2)?,
                    port_range: row.get(3)?,
                    success_score: row.get(4)?,
                    times_used: row.get(5)?,
                    last_verified: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })?;

            for method in rows {
                methods.push(method?);
            }
        }

        Ok(methods)
    }

    /// Get all cached methods
    pub fn get_all_methods(&self) -> SqliteResult<Vec<CachedMethod>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, method_name, target_pattern, port_range, success_score, times_used, last_verified, created_at
            FROM method_cache
            ORDER BY success_score DESC, times_used DESC",
        )?;

        let methods = stmt.query_map([], |row| {
            Ok(CachedMethod {
                id: row.get(0)?,
                method_name: row.get(1)?,
                target_pattern: row.get(2)?,
                port_range: row.get(3)?,
                success_score: row.get(4)?,
                times_used: row.get(5)?,
                last_verified: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;

        methods.collect()
    }

    /// Update success score for a method
    #[allow(dead_code)]
    pub fn update_success_score(&self, id: i64, score_delta: f64) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE method_cache SET success_score = success_score + ?1, last_verified = CURRENT_TIMESTAMP WHERE id = ?2",
            rusqlite::params![score_delta, id],
        )?;
        Ok(())
    }

    /// Delete a method by ID
    #[allow(dead_code)]
    pub fn delete_method(&self, id: i64) -> SqliteResult<()> {
        self.conn.execute("DELETE FROM method_cache WHERE id = ?1", rusqlite::params![id])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        crate::db::schema::create_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn test_cache_method() {
        let conn = test_db();
        let mc = MethodCache::new(&conn);

        mc.cache_method("tcp_scan", Some("192.168.%"), Some("1-1024")).unwrap();

        let methods = mc.get_all_methods().unwrap();
        assert!(!methods.is_empty());
        assert_eq!(methods[0].method_name, "tcp_scan");
    }

    #[test]
    fn test_get_cached_methods_empty() {
        let conn = test_db();
        let mc = MethodCache::new(&conn);

        let methods = mc.get_cached_methods(None).unwrap();
        assert!(methods.is_empty());
    }
}