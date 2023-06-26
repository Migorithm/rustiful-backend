#[cfg(test)]
pub mod components {
    use std::pin::Pin;

    use crate::adapters::database::{AtomicConnection, Connection};
    use dotenv::dotenv;
    use futures::Future;
    pub async fn get_connection() -> AtomicConnection {
        dotenv().unwrap();

        Connection::new().await.unwrap()
    }
    pub async fn tear_down() {
        let connection = get_connection().await;
        sqlx::query("TRUNCATE community_board, community_comment, auth_account, auth_token_stat,service_outbox")
            .execute(&connection.read().await.pool)
            .await
            .unwrap();
    }

    pub async fn run_test<T>(test: T) -> ()
    where
        T: FnOnce() -> Pin<Box<dyn Future<Output = ()>>>,
    {
        dotenv().unwrap();
        test().await;
        tear_down().await;
    }
}
