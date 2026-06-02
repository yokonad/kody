//! Scan history - track scan sessions for later reference

use rusqlite::{Connection, Result as SqliteResult};

/// A scan record from history
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ScanRecord {
    pub id: i64,
    pub target: String,
    pub scan_type: String,
    pub ports_scanned: Option<String>,
    pub vulnerabilities_found: i64,
    pub ai_used: bool,
    pub duration_ms: Option<i64>,
    pub timestamp: String,
}

/// Scan history manager
pub struct ScanHistory<'a> {
    conn: &'a Connection,
}

impl<'a> ScanHistory<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Record a completed scan
    pub fn record_scan(
        &self,
        target: &str,
        scan_type: &str,
        ports_scanned: Option<&str>,
        vulnerabilities_found: i64,
        ai_used: bool,
        duration_ms: Option<i64>,
    ) -> SqliteResult<()> {
        self.conn.execute(
            "INSERT INTO scan_history (target, scan_type, ports_scanned, vulnerabilities_found, ai_used, duration_ms)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![target, scan_type, ports_scanned, vulnerabilities_found, ai_used, duration_ms],
        )?;
        Ok(())
    }

    /// Get recent scans with limit
    #[allow(dead_code)]
    pub fn get_recent_scans(&self, limit: u32) -> SqliteResult<Vec<ScanRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, target, scan_type, ports_scanned, vulnerabilities_found, ai_used, duration_ms, timestamp
            FROM scan_history
            ORDER BY timestamp DESC
            LIMIT ?1",
        )?;

        let records = stmt.query_map([limit], |row| {
            Ok(ScanRecord {
                id: row.get(0)?,
                target: row.get(1)?,
                scan_type: row.get(2)?,
                ports_scanned: row.get(3)?,
                vulnerabilities_found: row.get(4)?,
                ai_used: row.get::<_, i64>(5)? != 0,
                duration_ms: row.get(6)?,
                timestamp: row.get(7)?,
            })
        })?;

        records.collect()
    }

    /// Get scans by target
    #[allow(dead_code)]
    pub fn get_scans_by_target(&self, target: &str) -> SqliteResult<Vec<ScanRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, target, scan_type, ports_scanned, vulnerabilities_found, ai_used, duration_ms, timestamp
            FROM scan_history
            WHERE target = ?1
            ORDER BY timestamp DESC",
        )?;

        let records = stmt.query_map(rusqlite::params![target], |row| {
            Ok(ScanRecord {
                id: row.get(0)?,
                target: row.get(1)?,
                scan_type: row.get(2)?,
                ports_scanned: row.get(3)?,
                vulnerabilities_found: row.get(4)?,
                ai_used: row.get::<_, i64>(5)? != 0,
                duration_ms: row.get(6)?,
                timestamp: row.get(7)?,
            })
        })?;

        records.collect()
    }

    /// Get total scan count
    #[allow(dead_code)]
    pub fn count(&self) -> SqliteResult<i64> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM scan_history",
            [],
            |row| row.get(0),
        )
    }

    /// Delete old scans (keep last N)
    #[allow(dead_code)]
    pub fn prune_old_scans(&self, keep_last: u32) -> SqliteResult<usize> {
        let deleted = self.conn.execute(
            "DELETE FROM scan_history WHERE id NOT IN (
                SELECT id FROM scan_history ORDER BY timestamp DESC LIMIT ?1
            )",
            [keep_last],
        )?;
        Ok(deleted)
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
    fn test_record_scan() {
        let conn = test_db();
        let sh = ScanHistory::new(&conn);

        sh.record_scan(
            "192.168.1.1",
            "scan",
            Some("1-1024"),
            3,
            true,
            Some(1500),
        ).unwrap();

        let count = sh.count().unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn test_get_recent_scans() {
        let conn = test_db();
        let sh = ScanHistory::new(&conn);

        sh.record_scan("192.168.1.1", "scan", None, 1, false, None).unwrap();
        sh.record_scan("192.168.1.2", "auto-scan", None, 2, true, None).unwrap();
        sh.record_scan("192.168.1.3", "map-hidden", None, 0, false, None).unwrap();

        let recent = sh.get_recent_scans(2).unwrap();
        assert_eq!(recent.len(), 2);
    }

    #[test]
    fn test_count() {
        let conn = test_db();
        let sh = ScanHistory::new(&conn);

        assert_eq!(sh.count().unwrap(), 0);
    }
}