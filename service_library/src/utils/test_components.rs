#[cfg(test)]
pub mod components {

    use dotenv::dotenv;
    use futures::Future;

    use crate::bootstrap::connection_pool;

    pub async fn tear_down() {
        let pool = connection_pool().await;
        sqlx::query("TRUNCATE community_board, community_comment, auth_account, auth_token_stat,service_outbox")
            .execute(pool)
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
