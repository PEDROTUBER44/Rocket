use deadpool_postgres::{Client, Transaction};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::Mutex;
use tokio_postgres::Statement;

use crate::error::{AppError, Result};

/// A thread-safe, asynchronous cache for prepared statements.
#[derive(Clone)]
pub struct StatementCache {
    cache: Arc<Mutex<HashMap<String, Statement>>>,
}

impl StatementCache {
    /// Creates a new, empty `StatementCache`.
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Retrieves a prepared statement from the cache, preparing it if it doesn't exist.
    pub async fn get_or_prepare_client(&self, client: &Client, query: &str) -> Result<Statement> {
        let mut cache = self.cache.lock().await;

        if let Some(statement) = cache.get(query) {
            return Ok(statement.clone());
        }

        let statement = client
            .prepare(query)
            .await
            .map_err(AppError::from)?;

        cache.insert(query.to_string(), statement.clone());

        Ok(statement)
    }

    /// Retrieves a prepared statement from the cache within a transaction.
    pub async fn get_or_prepare_transaction<'a>(&self, transaction: &Transaction<'a>, query: &str) -> Result<Statement> {
        let mut cache = self.cache.lock().await;

        if let Some(statement) = cache.get(query) {
            return Ok(statement.clone());
        }

        let statement = transaction
            .prepare(query)
            .await
            .map_err(AppError::from)?;

        cache.insert(query.to_string(), statement.clone());

        Ok(statement)
    }
}
