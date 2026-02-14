mod connection;

pub use connection::*;

#[derive(Clone, Debug)]
pub struct TableInfo {
    pub name: String,
    pub schema: String,
}

#[derive(Clone, Debug)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected_rows: u64,
}

impl QueryResult {
    pub fn empty() -> Self {
        Self {
            columns: vec![],
            rows: vec![],
            affected_rows: 0,
        }
    }
}
