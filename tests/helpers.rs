#[cfg(test)]
pub mod functions {

    use std::sync::Arc;

    use futures::Future;
    use library::adapters::database::Executor;
    use library::adapters::repositories::{Repository, TRepository};

    use dotenv::dotenv;
    use library::bootstrap::connection_pool;
    use library::domain::board::entity::{Board, BoardState};

    use library::domain::board::BoardAggregate;
    use library::domain::builder::{Buildable, Builder};

    use tokio::sync::RwLock;
    use uuid::Uuid;

    pub async fn tear_down() {
        let pool = connection_pool().await;
        sqlx::query("TRUNCATE community_board, community_comment, auth_account, auth_token_stat,service_outbox")
            .execute(pool)
            .await
            .unwrap();
    }

    pub fn board_repository_helper(executor: Arc<RwLock<Executor>>) -> Repository<BoardAggregate> {
        Repository::new(executor)
    }

    pub fn board_create_helper(state: BoardState) -> BoardAggregate {
        let builder = BoardAggregate::builder();
        builder
            .take_board(Board::new(
                Uuid::new_v4(),
                "Let's try it",
                "Is this fun?",
                state,
            ))
            .build()
    }

    pub async fn run_test<T>(test: T)
    where
        T: Future<Output = ()>,
    {
        dotenv().unwrap();
        Box::pin(test).await;
        tear_down().await;
    }
}
