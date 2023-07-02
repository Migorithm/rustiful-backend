use crate::utils::ApplicationError;
use crate::{domain::Message, utils::ApplicationResult};
use std::sync::OnceLock;
use std::{env, mem, sync::Arc};

use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    Postgres, Transaction,
};

use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::RwLock;

pub type AtomicContextManager = Arc<RwLock<ContextManager>>;

// ! Task Local ContextManager!
pub struct ContextManager {
    pub pool: &'static PgPool,
    pub sender: UnboundedSender<Box<dyn Message>>,
}

impl ContextManager {
    //TODO Creation of ContextManager - Need to be revisted
    pub async fn new(sender: UnboundedSender<Box<dyn Message>>) -> Arc<RwLock<Self>> {
        let pool = connection_pool().await;
        Arc::new(RwLock::new(Self { pool, sender }))
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

pub async fn connection_pool() -> &'static PgPool {
    let url = &env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let p = match POOL.get() {
        None => {
            let pool = PgPoolOptions::new()
                .max_connections(30)
                .connect(url)
                .await
                .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))
                .unwrap();
            POOL.get_or_init(|| pool)
        }
        Some(pool) => pool,
    };
    p
}

static POOL: OnceLock<PgPool> = OnceLock::new();
