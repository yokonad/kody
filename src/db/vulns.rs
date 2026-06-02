//! Vulnerability cache - store and retrieve discovered vulnerability patterns

use rusqlite::{Connection, Result as SqliteResult};

/// A cached vulnerability with details
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CachedVuln {
    pub id: i64,
    pub cve_id: Option<String>,
    pub description: Option<String>,
    pub severity: Option<String>,
    pub affected_ports: Option<String>,
    pub verified: bool,
    pub discovered_at: String,
}

/// Vulnerability cache manager
pub struct VulnCache<'a> {
    conn: &'a Connection,
}

impl<'a> VulnCache<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    /// Cache a vulnerability discovered during scanning
    pub fn cache_vulnerability(
        &self,
        cve_id: Option<&str>,
        description: Option<&str>,
        severity: Option<&str>,
        affected_ports: Option<&str>,
    ) -> SqliteResult<()> {
        // If CVE is provided, check for duplicates
        if let Some(cve) = cve_id {
            let existing: Option<i64> = self.conn.query_row(
                "SELECT id FROM vulnerability_cache WHERE cve_id = ?1",
                rusqlite::params![cve],
                |row| row.get(0),
            ).ok();

            if existing.is_some() {
                // Update existing vulnerability
                self.conn.execute(
                    "UPDATE vulnerability_cache SET
                        description = COALESCE(?1, description),
                        severity = COALESCE(?2, severity),
                        affected_ports = COALESCE(?3, affected_ports),
                        verified = 1
                    WHERE cve_id = ?4",
                    rusqlite::params![description, severity, affected_ports, cve],
                )?;
                return Ok(());
            }
        }

        // Insert new vulnerability
        self.conn.execute(
            "INSERT INTO vulnerability_cache (cve_id, description, severity, affected_ports, verified)
            VALUES (?1, ?2, ?3, ?4, 1)",
            rusqlite::params![cve_id, description, severity, affected_ports],
        )?;

        Ok(())
    }

    /// Get all cached vulnerabilities
    #[allow(dead_code)]
    pub fn get_known_vulnerabilities(&self) -> SqliteResult<Vec<CachedVuln>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, cve_id, description, severity, affected_ports, verified, discovered_at
            FROM vulnerability_cache
            ORDER BY severity, cve_id",
        )?;

        let vulns = stmt.query_map([], |row| {
            Ok(CachedVuln {
                id: row.get(0)?,
                cve_id: row.get(1)?,
                description: row.get(2)?,
                severity: row.get(3)?,
                affected_ports: row.get(4)?,
                verified: row.get::<_, i64>(5)? != 0,
                discovered_at: row.get(6)?,
            })
        })?;

        vulns.collect()
    }

    /// Get vulnerabilities by severity
    #[allow(dead_code)]
    pub fn get_by_severity(&self, severity: &str) -> SqliteResult<Vec<CachedVuln>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, cve_id, description, severity, affected_ports, verified, discovered_at
            FROM vulnerability_cache
            WHERE severity = ?1
            ORDER BY cve_id",
        )?;

        let vulns = stmt.query_map([severity], |row| {
            Ok(CachedVuln {
                id: row.get(0)?,
                cve_id: row.get(1)?,
                description: row.get(2)?,
                severity: row.get(3)?,
                affected_ports: row.get(4)?,
                verified: row.get::<_, i64>(5)? != 0,
                discovered_at: row.get(6)?,
            })
        })?;

        vulns.collect()
    }

    /// Get vulnerability by CVE ID
    #[allow(dead_code)]
    pub fn get_by_cve(&self, cve_id: &str) -> SqliteResult<Option<CachedVuln>> {
        let result = self.conn.query_row(
            "SELECT id, cve_id, description, severity, affected_ports, verified, discovered_at
            FROM vulnerability_cache
            WHERE cve_id = ?1",
            [cve_id],
            |row| {
                Ok(CachedVuln {
                    id: row.get(0)?,
                    cve_id: row.get(1)?,
                    description: row.get(2)?,
                    severity: row.get(3)?,
                    affected_ports: row.get(4)?,
                    verified: row.get::<_, i64>(5)? != 0,
                    discovered_at: row.get(6)?,
                })
            },
        );

        match result {
            Ok(vuln) => Ok(Some(vuln)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    /// Mark a vulnerability as verified/unverified
    #[allow(dead_code)]
    pub fn set_verified(&self, id: i64, verified: bool) -> SqliteResult<()> {
        self.conn.execute(
            "UPDATE vulnerability_cache SET verified = ?1 WHERE id = ?2",
            [if verified { 1 } else { 0 }, id],
        )?;
        Ok(())
    }

    /// Get count of cached vulnerabilities
    pub fn count(&self) -> SqliteResult<i64> {
        self.conn.query_row(
            "SELECT COUNT(*) FROM vulnerability_cache",
            [],
            |row| row.get(0),
        )
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
    fn test_cache_vulnerability() {
        let conn = test_db();
        let vc = VulnCache::new(&conn);

        vc.cache_vulnerability(
            Some("CVE-2023-38408"),
            Some("OpenSSH vulnerability"),
            Some("Medium"),
            Some("22"),
        ).unwrap();

        let vulns = vc.get_known_vulnerabilities().unwrap();
        assert!(!vulns.is_empty());
        assert_eq!(vulns[0].cve_id.as_deref(), Some("CVE-2023-38408"));
    }

    #[test]
    fn test_get_by_severity() {
        let conn = test_db();
        let vc = VulnCache::new(&conn);

        vc.cache_vulnerability(Some("CVE-1"), Some("Test 1"), Some("Critical"), None).unwrap();
        vc.cache_vulnerability(Some("CVE-2"), Some("Test 2"), Some("Low"), None).unwrap();

        let critical = vc.get_by_severity("Critical").unwrap();
        assert_eq!(critical.len(), 1);
    }

    #[test]
    fn test_count() {
        let conn = test_db();
        let vc = VulnCache::new(&conn);

        let count = vc.count().unwrap();
        assert_eq!(count, 0);

        vc.cache_vulnerability(Some("CVE-TEST"), Some("Test"), Some("High"), None).unwrap();
        assert_eq!(vc.count().unwrap(), 1);
    }
}