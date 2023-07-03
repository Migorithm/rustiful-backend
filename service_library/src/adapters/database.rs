use crate::bootstrap::connection_pool;
use crate::utils::ApplicationError;
use crate::{domain::Message, utils::ApplicationResult};
use std::collections::VecDeque;

use std::{mem, sync::Arc};

use sqlx::{postgres::PgPool, Postgres, Transaction};

use tokio::sync::RwLock;

pub type AtomicContextManager = Arc<RwLock<ContextManager>>;

/// Task Local Context Manager
/// This is called for every time Messagebus.handle is invoked within which it manages events raised in service.
/// It spawns out Executor that manages transaction.
pub struct ContextManager {
    pub pool: &'static PgPool,
    pub events: VecDeque<Box<dyn Message>>,
}

impl ContextManager {
    //TODO Creation of ContextManager - Need to be revisted
    pub async fn new() -> Arc<RwLock<Self>> {
        let pool = connection_pool().await;
        Arc::new(RwLock::new(Self {
            pool,
            events: Default::default(),
        }))
    }
    pub fn executor(&self) -> Arc<RwLock<Executor>> {
        RwLock::new(Executor::new(self.pool)).into()
    }
    pub fn events(&mut self) -> VecDeque<Box<dyn Message>> {
        mem::take(&mut self.events)
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
