use crate::config::Config;
use anyhow::Result;
use rusqlite::{params, Connection, Result as SqlResult};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    pub id: i64,
    pub name: String,
    pub version: String,
    pub runtime: String,
    pub r#type: String, // "template", "skill", "pkg"
    pub source: String,
    pub cache_path: String,
    pub checksum: String,
    pub size_bytes: Option<i64>,
    pub cached_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Template {
    pub name: String,
    pub category: String,
    pub description: String,
    pub version: String,
    pub source: String,
    pub files_count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub description: String,
    pub version: String,
    pub source: String,
    pub path: String,
}

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(config: &Config) -> Result<Self> {
        let db_path = config.cache_path().join("onpkg.db");
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let conn = Connection::open(&db_path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS packages (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL,
                version     TEXT NOT NULL,
                runtime     TEXT NOT NULL DEFAULT 'npm',
                type        TEXT NOT NULL DEFAULT 'pkg',
                source      TEXT NOT NULL DEFAULT '',
                cache_path  TEXT NOT NULL,
                checksum    TEXT NOT NULL,
                size_bytes  INTEGER,
                cached_at   DATETIME DEFAULT CURRENT_TIMESTAMP,
                UNIQUE(name, version, runtime, type)
            );

            CREATE TABLE IF NOT EXISTS templates (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,
                category    TEXT NOT NULL DEFAULT 'general',
                description TEXT NOT NULL DEFAULT '',
                version     TEXT NOT NULL DEFAULT '1.0.0',
                source      TEXT NOT NULL DEFAULT '',
                files_count INTEGER DEFAULT 0,
                installed_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS skills (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT NOT NULL UNIQUE,
                description TEXT NOT NULL DEFAULT '',
                version     TEXT NOT NULL DEFAULT '1.0.0',
                source      TEXT NOT NULL DEFAULT '',
                path        TEXT NOT NULL,
                installed_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );

            CREATE TABLE IF NOT EXISTS config (
                key   TEXT PRIMARY KEY,
                value TEXT
            );",
        )?;

        let integrity: String = conn.query_row("PRAGMA integrity_check", [], |row| row.get(0))?;
        if integrity != "ok" {
            return Err(anyhow::anyhow!("DB integrity check failed: {}", integrity));
        }

        Ok(Self { conn })
    }

    // ── Packages ────────────────────────────────────────────────────────

    pub fn insert_package(&self, pkg: &Package) -> Result<()> {
        self.conn.execute(
            "INSERT OR IGNORE INTO packages
                (name, version, runtime, type, source, cache_path, checksum, size_bytes, cached_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                pkg.name,
                pkg.version,
                pkg.runtime,
                pkg.r#type,
                pkg.source,
                pkg.cache_path,
                pkg.checksum,
                pkg.size_bytes,
                pkg.cached_at
            ],
        )?;
        Ok(())
    }

    pub fn list_packages(&self, runtime: Option<&str>) -> Result<Vec<Package>> {
        let mut results = Vec::new();
        let sql = if let Some(rt) = runtime {
            let mut stmt = self.conn.prepare(
                "SELECT id, name, version, runtime, type, source, cache_path, checksum, size_bytes, cached_at
                 FROM packages WHERE runtime = ?1 ORDER BY cached_at DESC"
            )?;
            let rows = stmt.query_map([rt], |row| row_to_package(row))?;
            for row in rows {
                results.push(row.map_err(|e| anyhow::anyhow!("{}", e))?);
            }
            results
        } else {
            let mut stmt = self.conn.prepare(
                "SELECT id, name, version, runtime, type, source, cache_path, checksum, size_bytes, cached_at
                 FROM packages ORDER BY cached_at DESC"
            )?;
            let rows = stmt.query_map([], |row| row_to_package(row))?;
            for row in rows {
                results.push(row.map_err(|e| anyhow::anyhow!("{}", e))?);
            }
            results
        };
        Ok(sql)
    }

    pub fn delete_package(&self, name: &str, runtime: &str) -> Result<usize> {
        let count = self.conn.execute(
            "DELETE FROM packages WHERE name = ?1 AND runtime = ?2",
            params![name, runtime],
        )?;
        Ok(count)
    }

    pub fn count_packages(&self) -> Result<i64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM packages", [], |row| row.get(0))?;
        Ok(count)
    }

    // ── Templates ───────────────────────────────────────────────────────

    pub fn insert_template(&self, tmpl: &Template) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO templates (name, category, description, version, source, files_count)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![tmpl.name, tmpl.category, tmpl.description, tmpl.version, tmpl.source, tmpl.files_count],
        )?;
        Ok(())
    }

    pub fn list_templates(&self) -> Result<Vec<Template>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, category, description, version, source, files_count
             FROM templates ORDER BY name",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(Template {
                name: row.get(0)?,
                category: row.get(1)?,
                description: row.get(2)?,
                version: row.get(3)?,
                source: row.get(4)?,
                files_count: row.get(5)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_template(&self, name: &str) -> Result<Option<Template>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, category, description, version, source, files_count
             FROM templates WHERE name = ?1",
        )?;
        match stmt.query_row(params![name], |row| {
            Ok(Template {
                name: row.get(0)?,
                category: row.get(1)?,
                description: row.get(2)?,
                version: row.get(3)?,
                source: row.get(4)?,
                files_count: row.get(5)?,
            })
        }) {
            Ok(t) => Ok(Some(t)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("DB query error: {}", e)),
        }
    }

    pub fn delete_template(&self, name: &str) -> Result<usize> {
        let count = self
            .conn
            .execute("DELETE FROM templates WHERE name = ?1", params![name])?;
        Ok(count)
    }

    // ── Skills ──────────────────────────────────────────────────────────

    pub fn insert_skill(&self, skill: &Skill) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO skills (name, description, version, source, path)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                skill.name,
                skill.description,
                skill.version,
                skill.source,
                skill.path
            ],
        )?;
        Ok(())
    }

    pub fn list_skills(&self) -> Result<Vec<Skill>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, description, version, source, path FROM skills ORDER BY name")?;
        let rows = stmt.query_map([], |row| {
            Ok(Skill {
                name: row.get(0)?,
                description: row.get(1)?,
                version: row.get(2)?,
                source: row.get(3)?,
                path: row.get(4)?,
            })
        })?;
        let mut results = Vec::new();
        for row in rows {
            results.push(row?);
        }
        Ok(results)
    }

    pub fn get_skill(&self, name: &str) -> Result<Option<Skill>> {
        let mut stmt = self.conn.prepare(
            "SELECT name, description, version, source, path FROM skills WHERE name = ?1",
        )?;
        match stmt.query_row(params![name], |row| {
            Ok(Skill {
                name: row.get(0)?,
                description: row.get(1)?,
                version: row.get(2)?,
                source: row.get(3)?,
                path: row.get(4)?,
            })
        }) {
            Ok(s) => Ok(Some(s)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(anyhow::anyhow!("DB query error: {}", e)),
        }
    }

    pub fn delete_skill(&self, name: &str) -> Result<usize> {
        let count = self
            .conn
            .execute("DELETE FROM skills WHERE name = ?1", params![name])?;
        Ok(count)
    }
}

fn row_to_package(row: &rusqlite::Row) -> SqlResult<Package> {
    Ok(Package {
        id: row.get(0)?,
        name: row.get(1)?,
        version: row.get(2)?,
        runtime: row.get(3)?,
        r#type: row.get(4)?,
        source: row.get(5)?,
        cache_path: row.get(6)?,
        checksum: row.get(7)?,
        size_bytes: row.get(8)?,
        cached_at: row.get(9)?,
    })
}

impl Clone for Database {
    fn clone(&self) -> Self {
        let path = self.conn.path().expect("Failed to get db path");
        let conn = Connection::open(path).expect("Failed to clone db connection");
        Self { conn }
    }
}
