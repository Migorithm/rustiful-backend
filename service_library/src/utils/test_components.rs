#[cfg(test)]
pub mod components {
    use crate::adapters::database::{AtomicContextManager, ContextManager};
    use dotenv::dotenv;
    use futures::Future;
    pub async fn get_connection() -> AtomicContextManager {
        dotenv().unwrap();

        ContextManager::new().await.unwrap()
    }
    pub async fn tear_down() {
        let connection = get_connection().await;
        sqlx::query("TRUNCATE community_board, community_comment, auth_account, auth_token_stat,service_outbox")
            .execute(&*connection.read().await.pool)
            .await
            .unwrap();
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
