pub mod helpers;

#[cfg(test)]
pub mod database_tests {

    use crate::helpers::functions::*;
    use service_library::{
        adapters::repository::{Repository, TRepository},
        domain::board::{entity::BoardState, events::BoardEvent, BoardAggregate},
    };

    #[tokio::test]
    async fn test_connection() {
        let connection = get_connection().await;

        match sqlx::query("SELECT 1")
            .execute(&connection.read().await.pool)
            .await
        {
            Ok(_val) => (),
            Err(_e) => panic!("Test Fail!"),
        };
    }

    #[tokio::test]
    async fn test_transaction_commit() {
        let connection = get_connection().await;
        // TODO test under same connection.
        connection.write().await.begin().await.unwrap();
        // let trx: Transactions = connection.begin().await.unwrap();
        let mut board_repo: Repository<BoardAggregate, BoardEvent> =
            board_repository_helper(connection.clone()).await;

        let mut board_aggregate = board_create_helper(BoardState::Unpublished);
        let id = board_repo.add(&mut board_aggregate).await.unwrap();

        connection.write().await.commit().await.unwrap();

        //TODO Should exist

        let _board_aggregate = board_repo.get(&id).await.unwrap();

        tear_down().await;
    }

    #[tokio::test]
    async fn test_transaction_rollback() {
        let connection = get_connection().await;

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

        tear_down().await;
    }
}
