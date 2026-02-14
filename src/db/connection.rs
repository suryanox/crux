use anyhow::Result;
use sqlx::{Column, Row};

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
                        columns
                            .iter()
                            .map(|col| {
                                row.try_get::<String, _>(col.as_str())
                                    .or_else(|_| row.try_get::<i64, _>(col.as_str()).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<i32, _>(col.as_str()).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<f64, _>(col.as_str()).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<bool, _>(col.as_str()).map(|v| v.to_string()))
                                    .unwrap_or_else(|_| "NULL".to_string())
                            })
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
                        columns
                            .iter()
                            .map(|col| {
                                row.try_get::<String, _>(col.as_str())
                                    .or_else(|_| row.try_get::<i64, _>(col.as_str()).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<i32, _>(col.as_str()).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<f64, _>(col.as_str()).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<bool, _>(col.as_str()).map(|v| v.to_string()))
                                    .unwrap_or_else(|_| "NULL".to_string())
                            })
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
                        columns
                            .iter()
                            .map(|col| {
                                row.try_get::<String, _>(col.as_str())
                                    .or_else(|_| row.try_get::<i64, _>(col.as_str()).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<i32, _>(col.as_str()).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<f64, _>(col.as_str()).map(|v| v.to_string()))
                                    .or_else(|_| row.try_get::<bool, _>(col.as_str()).map(|v| v.to_string()))
                                    .unwrap_or_else(|_| "NULL".to_string())
                            })
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
