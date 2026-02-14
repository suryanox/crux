use anyhow::Result;
use sqlx::{Column, Row, TypeInfo, ValueRef};

use super::{QueryResult, TableInfo};

pub enum DatabaseConnection {
    Postgres(sqlx::PgPool),
    MySql(sqlx::MySqlPool),
    Sqlite(sqlx::SqlitePool),
}

impl DatabaseConnection {
    pub async fn connect(connection_string: &str) -> Result<Self> {
        if connection_string.starts_with("postgres://") || connection_string.starts_with("postgresql://") {
            let pool = sqlx::PgPool::connect(connection_string).await?;
            Ok(Self::Postgres(pool))
        } else if connection_string.starts_with("mysql://") {
            let pool = sqlx::MySqlPool::connect(connection_string).await?;
            Ok(Self::MySql(pool))
        } else if connection_string.starts_with("sqlite://") || connection_string.ends_with(".db") {
            let conn_str = if connection_string.starts_with("sqlite://") {
                connection_string.to_string()
            } else {
                format!("sqlite://{}", connection_string)
            };
            let pool = sqlx::SqlitePool::connect(&conn_str).await?;
            Ok(Self::Sqlite(pool))
        } else {
            Err(anyhow::anyhow!("Unsupported database type"))
        }
    }

    pub async fn get_tables(&self) -> Result<Vec<TableInfo>> {
        match self {
            Self::Postgres(pool) => {
                let rows = sqlx::query(
                    "SELECT table_schema, table_name FROM information_schema.tables 
                     WHERE table_schema NOT IN ('pg_catalog', 'information_schema') 
                     ORDER BY table_schema, table_name"
                )
                .fetch_all(pool)
                .await?;

                Ok(rows
                    .iter()
                    .map(|row| TableInfo {
                        schema: row.get("table_schema"),
                        name: row.get("table_name"),
                    })
                    .collect())
            }
            Self::MySql(pool) => {
                let rows = sqlx::query(
                    "SELECT table_schema, table_name FROM information_schema.tables 
                     WHERE table_schema NOT IN ('mysql', 'information_schema', 'performance_schema', 'sys') 
                     ORDER BY table_schema, table_name"
                )
                .fetch_all(pool)
                .await?;

                Ok(rows
                    .iter()
                    .map(|row| TableInfo {
                        schema: row.get("TABLE_SCHEMA"),
                        name: row.get("TABLE_NAME"),
                    })
                    .collect())
            }
            Self::Sqlite(pool) => {
                let rows = sqlx::query(
                    "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
                )
                .fetch_all(pool)
                .await?;

                Ok(rows
                    .iter()
                    .map(|row| TableInfo {
                        schema: "main".to_string(),
                        name: row.get("name"),
                    })
                    .collect())
            }
        }
    }

    pub async fn execute_query(&self, query: &str) -> Result<QueryResult> {
        match self {
            Self::Postgres(pool) => {
                let rows = sqlx::query(query).fetch_all(pool).await?;
                if rows.is_empty() {
                    return Ok(QueryResult::empty());
                }

                let columns: Vec<String> = rows[0]
                    .columns()
                    .iter()
                    .map(|c| c.name().to_string())
                    .collect();

                let data: Vec<Vec<String>> = rows
                    .iter()
                    .map(|row| {
                        (0..columns.len())
                            .map(|idx| extract_pg_value(row, idx))
                            .collect()
                    })
                    .collect();

                Ok(QueryResult {
                    columns,
                    rows: data,
                    affected_rows: rows.len() as u64,
                })
            }
            Self::MySql(pool) => {
                let rows = sqlx::query(query).fetch_all(pool).await?;
                if rows.is_empty() {
                    return Ok(QueryResult::empty());
                }

                let columns: Vec<String> = rows[0]
                    .columns()
                    .iter()
                    .map(|c| c.name().to_string())
                    .collect();

                let data: Vec<Vec<String>> = rows
                    .iter()
                    .map(|row| {
                        (0..columns.len())
                            .map(|idx| extract_mysql_value(row, idx))
                            .collect()
                    })
                    .collect();

                Ok(QueryResult {
                    columns,
                    rows: data,
                    affected_rows: rows.len() as u64,
                })
            }
            Self::Sqlite(pool) => {
                let rows = sqlx::query(query).fetch_all(pool).await?;
                if rows.is_empty() {
                    return Ok(QueryResult::empty());
                }

                let columns: Vec<String> = rows[0]
                    .columns()
                    .iter()
                    .map(|c| c.name().to_string())
                    .collect();

                let data: Vec<Vec<String>> = rows
                    .iter()
                    .map(|row| {
                        (0..columns.len())
                            .map(|idx| extract_sqlite_value(row, idx))
                            .collect()
                    })
                    .collect();

                Ok(QueryResult {
                    columns,
                    rows: data,
                    affected_rows: rows.len() as u64,
                })
            }
        }
    }
}

fn extract_pg_value(row: &sqlx::postgres::PgRow, idx: usize) -> String {
    let value_ref = row.try_get_raw(idx).ok();
    
    if let Some(vr) = value_ref {
        if vr.is_null() {
            return "NULL".to_string();
        }
        
        let type_info = vr.type_info().clone();
        let type_name = type_info.name();
        
        match type_name {
            "BOOL" => {
                if let Ok(v) = row.try_get::<bool, _>(idx) {
                    return v.to_string();
                }
            }
            "INT2" | "SMALLINT" | "SMALLSERIAL" => {
                if let Ok(v) = row.try_get::<i16, _>(idx) {
                    return v.to_string();
                }
            }
            "INT4" | "INT" | "INTEGER" | "SERIAL" => {
                if let Ok(v) = row.try_get::<i32, _>(idx) {
                    return v.to_string();
                }
            }
            "INT8" | "BIGINT" | "BIGSERIAL" => {
                if let Ok(v) = row.try_get::<i64, _>(idx) {
                    return v.to_string();
                }
            }
            "FLOAT4" | "REAL" => {
                if let Ok(v) = row.try_get::<f32, _>(idx) {
                    return v.to_string();
                }
            }
            "FLOAT8" | "DOUBLE PRECISION" => {
                if let Ok(v) = row.try_get::<f64, _>(idx) {
                    return v.to_string();
                }
            }
            "NUMERIC" | "DECIMAL" => {
                if let Ok(v) = row.try_get::<sqlx::types::BigDecimal, _>(idx) {
                    return v.to_string();
                }
                if let Ok(v) = row.try_get::<f64, _>(idx) {
                    return v.to_string();
                }
            }
            "TEXT" | "VARCHAR" | "CHAR" | "BPCHAR" | "NAME" => {
                if let Ok(v) = row.try_get::<String, _>(idx) {
                    return v;
                }
            }
            "UUID" => {
                if let Ok(v) = row.try_get::<sqlx::types::Uuid, _>(idx) {
                    return v.to_string();
                }
            }
            "DATE" => {
                if let Ok(v) = row.try_get::<sqlx::types::chrono::NaiveDate, _>(idx) {
                    return v.to_string();
                }
            }
            "TIME" | "TIMETZ" => {
                if let Ok(v) = row.try_get::<sqlx::types::chrono::NaiveTime, _>(idx) {
                    return v.to_string();
                }
            }
            "TIMESTAMP" => {
                if let Ok(v) = row.try_get::<sqlx::types::chrono::NaiveDateTime, _>(idx) {
                    return v.to_string();
                }
            }
            "TIMESTAMPTZ" => {
                if let Ok(v) = row.try_get::<sqlx::types::chrono::DateTime<sqlx::types::chrono::Utc>, _>(idx) {
                    return v.to_string();
                }
            }
            "JSON" | "JSONB" => {
                if let Ok(v) = row.try_get::<sqlx::types::JsonValue, _>(idx) {
                    return v.to_string();
                }
            }
            "BYTEA" => {
                if let Ok(v) = row.try_get::<Vec<u8>, _>(idx) {
                    return format!("\\x{}", hex::encode(v));
                }
            }
            "INET" | "CIDR" => {
                if let Ok(v) = row.try_get::<String, _>(idx) {
                    return v;
                }
            }
            _ => {}
        }
    }
    
    row.try_get::<String, _>(idx)
        .or_else(|_| row.try_get::<i64, _>(idx).map(|v| v.to_string()))
        .or_else(|_| row.try_get::<i32, _>(idx).map(|v| v.to_string()))
        .or_else(|_| row.try_get::<f64, _>(idx).map(|v| v.to_string()))
        .or_else(|_| row.try_get::<bool, _>(idx).map(|v| v.to_string()))
        .unwrap_or_else(|_| "NULL".to_string())
}

fn extract_mysql_value(row: &sqlx::mysql::MySqlRow, idx: usize) -> String {
    let value_ref = row.try_get_raw(idx).ok();
    
    if let Some(vr) = value_ref {
        if vr.is_null() {
            return "NULL".to_string();
        }
        
        let type_info = vr.type_info().clone();
        let type_name = type_info.name();
        
        match type_name {
            "BOOLEAN" | "TINYINT(1)" => {
                if let Ok(v) = row.try_get::<bool, _>(idx) {
                    return v.to_string();
                }
            }
            "TINYINT" => {
                if let Ok(v) = row.try_get::<i8, _>(idx) {
                    return v.to_string();
                }
            }
            "SMALLINT" => {
                if let Ok(v) = row.try_get::<i16, _>(idx) {
                    return v.to_string();
                }
            }
            "INT" | "MEDIUMINT" => {
                if let Ok(v) = row.try_get::<i32, _>(idx) {
                    return v.to_string();
                }
            }
            "BIGINT" => {
                if let Ok(v) = row.try_get::<i64, _>(idx) {
                    return v.to_string();
                }
            }
            "FLOAT" => {
                if let Ok(v) = row.try_get::<f32, _>(idx) {
                    return v.to_string();
                }
            }
            "DOUBLE" => {
                if let Ok(v) = row.try_get::<f64, _>(idx) {
                    return v.to_string();
                }
            }
            "DECIMAL" => {
                if let Ok(v) = row.try_get::<sqlx::types::BigDecimal, _>(idx) {
                    return v.to_string();
                }
            }
            "VARCHAR" | "CHAR" | "TEXT" | "TINYTEXT" | "MEDIUMTEXT" | "LONGTEXT" | "ENUM" | "SET" => {
                if let Ok(v) = row.try_get::<String, _>(idx) {
                    return v;
                }
            }
            "DATE" => {
                if let Ok(v) = row.try_get::<sqlx::types::chrono::NaiveDate, _>(idx) {
                    return v.to_string();
                }
            }
            "TIME" => {
                if let Ok(v) = row.try_get::<sqlx::types::chrono::NaiveTime, _>(idx) {
                    return v.to_string();
                }
            }
            "DATETIME" | "TIMESTAMP" => {
                if let Ok(v) = row.try_get::<sqlx::types::chrono::NaiveDateTime, _>(idx) {
                    return v.to_string();
                }
            }
            "JSON" => {
                if let Ok(v) = row.try_get::<sqlx::types::JsonValue, _>(idx) {
                    return v.to_string();
                }
            }
            "BLOB" | "TINYBLOB" | "MEDIUMBLOB" | "LONGBLOB" | "BINARY" | "VARBINARY" => {
                if let Ok(v) = row.try_get::<Vec<u8>, _>(idx) {
                    return format!("0x{}", hex::encode(v));
                }
            }
            _ => {}
        }
    }
    
    row.try_get::<String, _>(idx)
        .or_else(|_| row.try_get::<i64, _>(idx).map(|v| v.to_string()))
        .or_else(|_| row.try_get::<i32, _>(idx).map(|v| v.to_string()))
        .or_else(|_| row.try_get::<f64, _>(idx).map(|v| v.to_string()))
        .or_else(|_| row.try_get::<bool, _>(idx).map(|v| v.to_string()))
        .unwrap_or_else(|_| "NULL".to_string())
}

fn extract_sqlite_value(row: &sqlx::sqlite::SqliteRow, idx: usize) -> String {
    let value_ref = row.try_get_raw(idx).ok();
    
    if let Some(vr) = value_ref {
        if vr.is_null() {
            return "NULL".to_string();
        }
        
        let type_info = vr.type_info().clone();
        let type_name = type_info.name();
        
        match type_name {
            "INTEGER" => {
                if let Ok(v) = row.try_get::<i64, _>(idx) {
                    return v.to_string();
                }
            }
            "REAL" => {
                if let Ok(v) = row.try_get::<f64, _>(idx) {
                    return v.to_string();
                }
            }
            "TEXT" => {
                if let Ok(v) = row.try_get::<String, _>(idx) {
                    return v;
                }
            }
            "BLOB" => {
                if let Ok(v) = row.try_get::<Vec<u8>, _>(idx) {
                    return format!("X'{}'", hex::encode(v));
                }
            }
            "BOOLEAN" => {
                if let Ok(v) = row.try_get::<bool, _>(idx) {
                    return v.to_string();
                }
            }
            "DATE" => {
                if let Ok(v) = row.try_get::<sqlx::types::chrono::NaiveDate, _>(idx) {
                    return v.to_string();
                }
                if let Ok(v) = row.try_get::<String, _>(idx) {
                    return v;
                }
            }
            "DATETIME" | "TIMESTAMP" => {
                if let Ok(v) = row.try_get::<sqlx::types::chrono::NaiveDateTime, _>(idx) {
                    return v.to_string();
                }
                if let Ok(v) = row.try_get::<String, _>(idx) {
                    return v;
                }
            }
            _ => {}
        }
    }
    
    row.try_get::<String, _>(idx)
        .or_else(|_| row.try_get::<i64, _>(idx).map(|v| v.to_string()))
        .or_else(|_| row.try_get::<f64, _>(idx).map(|v| v.to_string()))
        .or_else(|_| row.try_get::<bool, _>(idx).map(|v| v.to_string()))
        .or_else(|_| row.try_get::<Vec<u8>, _>(idx).map(|v| format!("X'{}'", hex::encode(v))))
        .unwrap_or_else(|_| "NULL".to_string())
}
