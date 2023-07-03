use crate::bootstrap::connection_pool;
use crate::utils::ApplicationError;
use crate::{domain::Message, utils::ApplicationResult};

use std::{mem, sync::Arc};

use sqlx::{postgres::PgPool, Postgres, Transaction};

use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::{mpsc::channel, RwLock};

pub type AtomicContextManager = Arc<RwLock<ContextManager>>;

/// Task Local Context Manager
/// This is called for every time Messagebus.handle is invoked within which it manages events raised in service.
/// It spawns out Executor that manages transaction.
pub struct ContextManager {
    pub pool: &'static PgPool,

    pub sender: Sender<Box<dyn Message>>,
}

impl ContextManager {
    /// Creation of context manager returns context manager AND event receiver
    pub async fn new() -> (Arc<RwLock<Self>>, Receiver<Box<dyn Message>>) {
        let pool = connection_pool().await;
        let (sender, receiver) = channel(20);
        (Arc::new(RwLock::new(Self { pool, sender })), receiver)
    }
    pub fn executor(&self) -> Arc<RwLock<Executor>> {
        RwLock::new(Executor::new(self.pool)).into()
    }
}

#[derive(Debug)]
pub struct Executor {
    pub(crate) pool: &'static PgPool,
    pub(crate) transaction: Option<Transaction<'static, Postgres>>,
}

impl Executor {
    pub fn new(pool: &'static PgPool) -> Self {
        Self {
            pool,
            transaction: None,
        }
    }

    pub async fn begin(&mut self) -> ApplicationResult<()> {
        match self.transaction.as_mut() {
            None => {
                self.transaction = Some(
                    self.pool
                        .begin()
                        .await
                        .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?,
                )
            }
            Some(_trx) => {
                println!("Transaction Begun Already!");
                Err(ApplicationError::TransactionError)?
            }
        };
        Ok(())
    }

    pub async fn commit(&mut self) -> ApplicationResult<()> {
        if self.transaction.is_none() {
            panic!("Tranasction Has Not Begun!");
        };

        let trx = mem::take(&mut self.transaction).unwrap();
        trx.commit()
            .await
            .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;
        Ok(())
    }
    pub async fn rollback(&mut self) -> ApplicationResult<()> {
        if self.transaction.is_none() {
            panic!("Tranasction Has Not Begun!");
        };

        let trx = mem::take(&mut self.transaction).unwrap();
        trx.rollback()
            .await
            .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;
        Ok(())
    }

    pub fn transaction(&mut self) -> &mut Transaction<'static, Postgres> {
        match self.transaction.as_mut() {
            Some(trx) => trx,
            None => panic!("Transaction Has Not Begun!"),
        }
    }
    pub fn connection(&self) -> &PgPool {
        self.pool
    }
}
