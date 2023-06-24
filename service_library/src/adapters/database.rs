use std::{env, mem, sync::Arc};

use crate::utils::ApplicationError;

use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    Postgres, Transaction,
};
use tokio::sync::RwLock;

pub type AtomicConnection = Arc<RwLock<Connection>>;

#[derive(Debug)]
pub struct Connection {
    pub executor: Executor,
    pub pool: PgPool,
}

impl Connection {
    pub async fn new() -> Result<Arc<RwLock<Self>>, ApplicationError> {
        let url: &str = &env::var("DATABASE_URL").map_err(|_| ApplicationError::InvalidURL)?;

        let db_pool = PgPoolOptions::new()
            .max_connections(30)
            .connect(url)
            .await
            .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;
        Ok(Arc::new(RwLock::new(Self {
            pool: db_pool,
            executor: Executor::NotSet,
        })))
    }

    pub fn connection(&mut self) -> &mut Transaction<'static, Postgres> {
        self.executor.connection()
    }
    pub async fn begin(&mut self) -> Result<(), ApplicationError> {
        match &self.executor {
            Executor::NotSet => {
                let transaction = Executor::PgTransaction(
                    self.pool
                        .begin()
                        .await
                        .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?,
                );
                self.executor = transaction;

                Ok(())
            }
            Executor::PgTransaction(_) => {
                println!("Transaction Begun Already!");
                Err(ApplicationError::TransactionError)
            }
        }
    }

    pub async fn commit(&mut self) -> Result<(), ApplicationError> {
        if let Executor::PgTransaction(_t) = &mut self.executor {
            let old = mem::replace(&mut self.executor, Executor::NotSet);
            match old {
                Executor::PgTransaction(trx) => {
                    trx.commit()
                        .await
                        .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;
                    Ok(())
                }
                _ => panic!("Tranasction Has Not Begun!"),
            }
        } else {
            panic!("Tranasction Has Not Begun!")
        }
    }

    pub async fn rollback(&mut self) -> Result<(), ApplicationError> {
        if let Executor::PgTransaction(_t) = &mut self.executor {
            let old = mem::replace(&mut self.executor, Executor::NotSet);
            match old {
                Executor::PgTransaction(trx) => {
                    trx.rollback()
                        .await
                        .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))?;
                    Ok(())
                }
                _ => panic!("Tranasction Has Not Begun!"),
            }
        } else {
            panic!("Tranasction Has Not Begun!")
        }
    }
}

#[derive(Debug)]
pub enum Executor {
    NotSet,
    PgTransaction(Transaction<'static, Postgres>),
}

impl Executor {
    pub fn connection(&mut self) -> &mut Transaction<'static, Postgres> {
        match self {
            Self::PgTransaction(transaction) => transaction,
            Self::NotSet => panic!("Transaction Has Not Begun!"),
        }
    }
}
