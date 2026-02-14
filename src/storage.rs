use anyhow::Result;
use sqlx::sqlite::SqlitePool;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RecentConnection {
    pub id: i64,
    pub connection_string: String,
    pub display_name: String,
    pub last_used: String,
}

pub struct Storage {
    pool: SqlitePool,
}

impl Storage {
    pub async fn new() -> Result<Self> {
        let db_path = Self::get_db_path()?;
        
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        let db_url = format!("sqlite:{}?mode=rwc", db_path.display());
        let pool = SqlitePool::connect(&db_url).await?;
        
        let storage = Self { pool };
        storage.init_schema().await?;
        
        Ok(storage)
    }
    
    fn get_db_path() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".crux").join("crux.db"))
    }
    
    async fn init_schema(&self) -> Result<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS recent_connections (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                connection_string TEXT NOT NULL UNIQUE,
                display_name TEXT NOT NULL,
                last_used DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn add_connection(&self, connection_string: &str) -> Result<()> {
        let display_name = Self::generate_display_name(connection_string);
        
        sqlx::query(
            r#"
            INSERT INTO recent_connections (connection_string, display_name, last_used)
            VALUES (?, ?, CURRENT_TIMESTAMP)
            ON CONFLICT(connection_string) DO UPDATE SET last_used = CURRENT_TIMESTAMP
            "#,
        )
        .bind(connection_string)
        .bind(&display_name)
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn get_recent_connections(&self, limit: i32) -> Result<Vec<RecentConnection>> {
        let rows = sqlx::query_as::<_, (i64, String, String, String)>(
            r#"
            SELECT id, connection_string, display_name, datetime(last_used) as last_used
            FROM recent_connections
            ORDER BY last_used DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;
        
        Ok(rows
            .into_iter()
            .map(|(id, connection_string, display_name, last_used)| RecentConnection {
                id,
                connection_string,
                display_name,
                last_used,
            })
            .collect())
    }
    
    pub async fn delete_connection(&self, id: i64) -> Result<()> {
        sqlx::query("DELETE FROM recent_connections WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    fn generate_display_name(connection_string: &str) -> String {
        if connection_string.starts_with("postgres://") || connection_string.starts_with("postgresql://") {
            Self::parse_url_display_name(connection_string, "PostgreSQL")
        } else if connection_string.starts_with("mysql://") {
            Self::parse_url_display_name(connection_string, "MySQL")
        } else if connection_string.starts_with("sqlite://") {
            let path = connection_string.strip_prefix("sqlite://").unwrap_or(connection_string);
            format!("SQLite: {}", path.split('/').last().unwrap_or(path))
        } else if connection_string.ends_with(".db") {
            format!("SQLite: {}", connection_string.split('/').last().unwrap_or(connection_string))
        } else {
            connection_string.chars().take(40).collect()
        }
    }
    
    fn parse_url_display_name(url: &str, db_type: &str) -> String {
        if let Some(rest) = url.split("://").nth(1) {
            let without_auth = if let Some(at_pos) = rest.find('@') {
                &rest[at_pos + 1..]
            } else {
                rest
            };
            
            let parts: Vec<&str> = without_auth.split('/').collect();
            let host = parts.first().map(|h| {
                h.split(':').next().unwrap_or(h)
            }).unwrap_or("unknown");
            
            let database = parts.get(1).map(|d| {
                d.split('?').next().unwrap_or(d)
            }).unwrap_or("default");
            
            format!("{}: {}@{}", db_type, database, host)
        } else {
            format!("{}: {}", db_type, url.chars().take(30).collect::<String>())
        }
    }
}
