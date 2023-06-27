#[allow(unused)]
#[cfg(test)]
pub mod functions {

    use std::pin::Pin;
    use std::str::FromStr;

    use futures::Future;
    use service_library::adapters::database::{AtomicConnection, Connection};
    use service_library::adapters::repositories::{Repository, TRepository};

    use dotenv::dotenv;
    use service_library::domain::board::entity::{Board, BoardState, Comment};
    use service_library::domain::board::events::BoardEvent;
    use service_library::domain::board::BoardAggregate;
    use service_library::domain::builder::{Buildable, Builder};

    use uuid::Uuid;

    pub async fn tear_down() {
        let connection = Connection::new().await.unwrap();
        sqlx::query("TRUNCATE community_board, community_comment, auth_account, auth_token_stat,service_outbox")
            .execute(&connection.read().await.pool)
            .await
            .unwrap();
    }

    pub async fn board_repository_helper(
        connection: AtomicConnection,
    ) -> Repository<BoardAggregate, BoardEvent> {
        Repository::new(connection)
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
    pub fn comment_create_helper(id: &str) -> Comment {
        let uuidfied = Uuid::from_str(id).expect("Not Uuidfiable!");

        Comment::new(uuidfied, Uuid::new_v4(), "노잼")
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
