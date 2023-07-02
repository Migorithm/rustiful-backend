pub mod helpers;

#[cfg(test)]
pub mod database_tests {

    use crate::helpers::functions::*;
    use service_library::{
        adapters::{
            database::ContextManager,
            repositories::{Repository, TRepository},
        },
        domain::board::{entity::BoardState, BoardAggregate},
    };

    #[tokio::test]
    async fn test_connection() {
        run_test(async {
            let connection = ContextManager::new().await.unwrap();

            match sqlx::query("SELECT 1")
                .execute(connection.read().await.pool)
                .await
            {
                Ok(_val) => (),
                Err(_e) => panic!("Test Fail!"),
            };
        })
        .await;
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        run_test(async {
            let connection = ContextManager::new().await.unwrap();
            // TODO test under same connection.
            connection.write().await.begin().await.unwrap();
            // let trx: Transactions = connection.begin().await.unwrap();
            let mut board_repo: Repository<BoardAggregate> =
                board_repository_helper(connection.clone()).await;

            let mut board_aggregate = board_create_helper(BoardState::Unpublished);
            let id = board_repo.add(&mut board_aggregate).await.unwrap();

            connection.write().await.commit().await.unwrap();

            //TODO Should exist

            let _board_aggregate = board_repo.get(&id).await.unwrap();
        })
        .await;
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        run_test(async {
            let connection = ContextManager::new().await.unwrap();

            connection.write().await.begin().await.unwrap();

            // TODO test under same connection.

            let mut board_repo = board_repository_helper(connection.clone()).await;

            let mut board_aggregate = board_create_helper(BoardState::Unpublished);
            let id = board_repo.add(&mut board_aggregate).await.unwrap();

            connection.write().await.rollback().await.unwrap();

            //TODO Shouldn't exist
            if board_repo.get(&id).await.is_ok() {
                panic!("Shouldn't exist!")
            }
        })
        .await;
    }
}
