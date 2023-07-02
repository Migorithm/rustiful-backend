use crate::domain::Message;
use crate::utils::ApplicationError;
use std::{
    collections::VecDeque,
    env, mem,
    sync::{
        atomic::AtomicPtr,
        atomic::Ordering::{Acquire, Release},
        Arc,
    },
};

use sqlx::{
    postgres::{PgPool, PgPoolOptions},
    Postgres, Transaction,
};
use tokio::sync::RwLock;

pub type AtomicContextManager = Arc<RwLock<ContextManager>>;

// ! Task Local ContextManager!
pub struct ContextManager {
    pub executor: Executor,
    pub pool: &'static PgPool,
    pub events: VecDeque<Box<dyn Message>>,
}

impl ContextManager {
    //TODO Creation of ContextManager - Need to be revisted
    pub async fn new() -> Result<Arc<RwLock<Self>>, ApplicationError> {
        Ok(Arc::new(RwLock::new(Self {
            pool: connection_pool().await,
            executor: Executor::NotSet,
            events: Default::default(),
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
    pub fn events(&mut self) -> VecDeque<Box<dyn Message>> {
        mem::take(&mut self.events)
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

pub async fn connection_pool() -> &'static PgPool {
    static PTR: AtomicPtr<PgPool> = AtomicPtr::new(std::ptr::null_mut());
    let mut p = PTR.load(Acquire);

    if p.is_null() {
        let url = &env::var("DATABASE_URL").expect("DATABASE_URL must be set");

        p = Box::into_raw(Box::new(
            PgPoolOptions::new()
                .max_connections(30)
                .connect(url)
                .await
                .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))
                .unwrap(),
        ));
        if let Err(e) = PTR.compare_exchange(std::ptr::null_mut(), p, Release, Acquire) {
            // Safety: p comes from Box::into_raw right above
            // and wasn't whared with any other thread
            drop(unsafe { Box::from_raw(p) });
            p = e;
        }
    }
    // Safety: p is not null and points to a properly initialized value
    unsafe { &*p }
}
