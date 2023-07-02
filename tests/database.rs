pub mod helpers;

#[cfg(test)]
pub mod database_tests {

    use std::sync::Arc;

    use crate::helpers::functions::*;
    use service_library::{
        adapters::{
            database::{connection_pool, ContextManager, Executor},
            repositories::{Repository, TRepository},
        },
        domain::board::{entity::BoardState, BoardAggregate},
    };
    use tokio::sync::RwLock;

    #[tokio::test]
    async fn test_connection() {
        run_test(async {
            let pool = connection_pool().await;

            match sqlx::query("SELECT 1").execute(pool).await {
                Ok(_val) => (),
                Err(_e) => panic!("Test Fail!"),
            };
        })
        .await;
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        run_test(async {
            let pool = connection_pool().await;
            let executor = Arc::new(RwLock::new(Executor::new(pool)));
            // let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
            //     connection.read().await.executor(),
            // );

            executor.write().await.begin().await.unwrap();

            let mut board_repo: Repository<BoardAggregate> =
                board_repository_helper(executor.clone()).await;

            let mut board_aggregate = board_create_helper(BoardState::Unpublished);
            let id = board_repo.add(&mut board_aggregate).await.unwrap();

            executor.write().await.commit().await.unwrap();

            //TODO Should exist

            let _board_aggregate = board_repo.get(&id).await.unwrap();
        })
        .await;
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        run_test(async {
            let pool = connection_pool().await;
            let executor = Arc::new(RwLock::new(Executor::new(pool)));

            executor.write().await.begin().await.unwrap();
            // let trx: Transactions = connection.begin().await.unwrap();
            let mut board_repo: Repository<BoardAggregate> =
                board_repository_helper(executor.clone()).await;

            let mut board_aggregate = board_create_helper(BoardState::Unpublished);
            let id = board_repo.add(&mut board_aggregate).await.unwrap();

            executor.write().await.rollback().await.unwrap();

            //TODO Shouldn't exist
            if board_repo.get(&id).await.is_ok() {
                panic!("Shouldn't exist!")
            }
        })
        .await;
    }
}
