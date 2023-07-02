mod helpers;

#[cfg(test)]
mod repository_tests {
    use crate::helpers::functions::*;
    use service_library::adapters::database::ContextManager;
    use service_library::adapters::repositories::TRepository;

    use std::str::FromStr;

    use service_library::domain::board::commands::EditBoard;
    use service_library::domain::board::entity::BoardState;

    use service_library::domain::board::BoardAggregate;
    use service_library::domain::builder::{Buildable, Builder};

    use uuid::Uuid;

    #[tokio::test]
    async fn test_add_board() {
        run_test(async {
            let context_manager = ContextManager::new().await;

            let executor = context_manager.read().await.executor();

            '_transaction_block: {
                executor.write().await.begin().await.unwrap();

                let mut board_repo = board_repository_helper(executor.clone()).await;

                let mut board_aggregate = board_create_helper(BoardState::Unpublished);

                let id = board_repo.add(&mut board_aggregate).await.unwrap();
                assert_eq!(board_aggregate.board.id.to_string(), id);

                executor.write().await.commit().await.unwrap();
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_board() {
        run_test(async {
            let context_manager = ContextManager::new().await;
            let executor = context_manager.read().await.executor();

            let mut board_repo = board_repository_helper(executor.clone()).await;
            let id: String;

            '_tranasction_block: {
                executor.write().await.begin().await.unwrap();

                let mut board_aggregate = board_create_helper(BoardState::Unpublished);

                id = board_repo.add(&mut board_aggregate).await.unwrap();
                executor.write().await.commit().await.unwrap();
            }

            '_test_block: {
                let board_aggregate = board_repo.get(&id).await.unwrap();
                assert_eq!(board_aggregate.board.state(), "Unpublished");
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_get_board_with_different_state() {
        run_test(async {
            let context_manager = ContextManager::new().await;
            let executor = context_manager.read().await.executor();

            let mut board_repo = board_repository_helper(executor.clone()).await;
            let id: String;

            '_transaction_block: {
                executor.write().await.begin().await.unwrap();

                let mut board_aggregate = board_create_helper(BoardState::Published);

                id = board_repo.add(&mut board_aggregate).await.unwrap();
                executor.write().await.commit().await.unwrap();
            }

            '_test_block: {
                let board_aggregate = board_repo.get(&id).await.unwrap();

                assert_eq!(board_aggregate.board.state(), "Published");
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_delete_board() {
        run_test(async {
            let context_manager = ContextManager::new().await;
            let executor = context_manager.read().await.executor();

            let mut board_repo = board_repository_helper(executor.clone()).await;
            let id: String;

            '_transaction_block: {
                executor.write().await.begin().await.unwrap();

                let mut board_aggregate = board_create_helper(BoardState::Unpublished);
                assert_eq!(board_aggregate.board.state(), "Unpublished");
                id = board_repo.add(&mut board_aggregate).await.unwrap();
                executor.write().await.commit().await.unwrap();
            }

            '_transaction_block2: {
                executor.write().await.begin().await.unwrap();
                let mut board_aggregate = board_repo.get(&id).await.unwrap();
                board_aggregate.update_board(EditBoard {
                    id: Uuid::from_str(&id).unwrap(),
                    title: None,
                    content: None,
                    state: Some(BoardState::Deleted),
                });

                board_repo.update(&mut board_aggregate).await.unwrap();
                executor.write().await.commit().await.unwrap();
            }
            '_test_block3: {
                let board_aggregate = board_repo.get(&id).await.unwrap();
                assert_eq!(board_aggregate.board.state(), "Deleted");
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_update_board() {
        run_test(async {
            let context_manager = ContextManager::new().await;
            let executor = context_manager.read().await.executor();
            let mut board_repo = board_repository_helper(executor.clone()).await;
            let id: String;

            let existing_content: String;

            '_transaction_block: {
                executor.write().await.begin().await.unwrap();

                let mut board_aggregate = board_create_helper(BoardState::Unpublished);
                existing_content = board_aggregate.board.content.clone();

                id = board_repo.add(&mut board_aggregate).await.unwrap();

                executor.write().await.commit().await.unwrap();
            }

            '_transaction_block2: {
                executor.write().await.begin().await.unwrap();

                let mut initial_board_aggregate = board_repo.get(&id).await.unwrap();
                let initial_board = &mut initial_board_aggregate.board;

                '_test_block: {
                    assert_eq!(initial_board.state(), "Unpublished");
                    assert_eq!(initial_board.version, 0);
                }
                let new_author = Uuid::new_v4();
                initial_board.author = new_author;
                initial_board.content = "Something else".to_string();
                board_repo
                    .update(&mut initial_board_aggregate)
                    .await
                    .unwrap();

                executor.write().await.commit().await.unwrap();

                '_test_block: {
                    let board_aggregate = board_repo.get(&id).await.unwrap();
                    let updated_board = board_aggregate.board;

                    assert_eq!(updated_board.author, new_author);
                    assert_ne!(updated_board.content, existing_content);
                    assert_eq!(updated_board.version, 1);
                }
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_create_comment() {
        run_test(async {
            let context_manager = ContextManager::new().await;
            let executor = context_manager.read().await.executor();
            let mut board_repo = board_repository_helper(executor.clone()).await;
            let mut board_aggregate: BoardAggregate;
            let id: String;

            '_tranasction_block: {
                executor.write().await.begin().await.unwrap();
                board_aggregate = board_create_helper(BoardState::Unpublished);
                id = board_repo.add(&mut board_aggregate).await.unwrap();
                executor.write().await.commit().await.unwrap();
            }

            '_transaction_block2: {
                let comment = comment_create_helper(&id);

                let mut board_builder = BoardAggregate::builder();
                board_builder = board_builder
                    .take_board(board_aggregate.board)
                    .take_comments(vec![comment]);

                executor.write().await.begin().await.unwrap();
                board_repo.update(&mut board_builder.build()).await.unwrap();
                executor.write().await.commit().await.unwrap();
            }

            '_test_block: {
                let board_aggregate = board_repo.get(&id).await.unwrap();

                assert_eq!(board_aggregate.board.version, 1);
                assert_eq!(board_aggregate.comments.len(), 1);
            }
        })
        .await;
    }
}
