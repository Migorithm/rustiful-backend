
mod helpers;

#[cfg(test)]
mod repository_tests {

    use std::str::FromStr;

    use crate::helpers::functions;

    use service_library::adapters::repository::{Repository, TRepository};

    use service_library::domain::board::entity::BoardState;
    
    use service_library::domain::board::events::BoardEvent;
    use service_library::domain::board::BoardAggregate;
    use service_library::domain::builder::{Buildable, Builder};
    use service_library::domain::commands::ApplicationCommand;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_add_board() {
        let connection = functions::get_connection().await;
        let mut board_repo = functions::board_repository_helper(connection.clone()).await;

        '_transaction_block: {
            connection.write().await.begin().await.unwrap();

            let mut board_aggregate = functions::board_create_helper(BoardState::Unpublished);

            let id = board_repo.add(&mut board_aggregate).await.unwrap();
            assert_eq!(board_aggregate.board.id.to_string(), id);

            connection.write().await.commit().await.unwrap();
        }

        functions::tear_down().await;
    }

    #[tokio::test]
    async fn test_get_board() {
        let connection = functions::get_connection().await;
        let mut board_repo = functions::board_repository_helper(connection.clone()).await;
        let id: String;

        '_tranasction_block: {
            connection.write().await.begin().await.unwrap();

            let mut board = functions::board_create_helper(BoardState::Unpublished);

            id = board_repo.add(&mut board).await.unwrap();
            connection.write().await.commit().await.unwrap();
        }

        '_test_block: {
            let board_aggregate = board_repo.get(&id).await.unwrap();
            assert_eq!(board_aggregate.board.state(), "Unpublished");
        }
        functions::tear_down().await;
    }

    #[tokio::test]
    async fn test_get_board_with_different_state() {
        let connection = functions::get_connection().await;
        let mut board_repo = functions::board_repository_helper(connection.clone()).await;
        let id: String;

        '_transaction_block: {
            connection.write().await.begin().await.unwrap();

            let mut board = functions::board_create_helper(BoardState::Published);

            id = board_repo.add(&mut board).await.unwrap();

            connection.write().await.commit().await.unwrap();
        }

        '_test_block: {
            let board_aggregate = board_repo.get(&id).await.unwrap();

            assert_eq!(board_aggregate.board.state(), "Published");
        }
        functions::tear_down().await;
    }

    #[tokio::test]
    async fn test_delete_board() {
        let connection = functions::get_connection().await;
        let mut board_repo: Repository<BoardAggregate, BoardEvent> =
            functions::board_repository_helper(connection.clone()).await;

        let id: String;
        '_transaction_block: {
            connection.write().await.begin().await.unwrap();

            let mut board_aggregate = functions::board_create_helper(BoardState::Unpublished);
            assert_eq!(board_aggregate.board.state(), "Unpublished");
            id = board_repo.add(&mut board_aggregate).await.unwrap();
            connection.write().await.commit().await.unwrap();
        }

        '_transaction_block2: {
            connection.write().await.begin().await.unwrap();
            let mut board_aggregate = board_repo.get(&id).await.unwrap();
            board_aggregate
                .execute(ApplicationCommand::EditBoard {
                    id: Uuid::from_str(&id).unwrap(),
                    title: None,
                    content: None,
                    state: Some(BoardState::Deleted),
                })
                .unwrap();
            board_repo.update(board_aggregate).await.unwrap();
            connection.write().await.commit().await.unwrap();
        }
        '_test_block3: {
            let board_aggregate = board_repo.get(&id).await.unwrap();
            assert_eq!(board_aggregate.board.state(), "Deleted");
        }
        functions::tear_down().await;
    }

    #[tokio::test]
    async fn test_update_board() {
        let connection = functions::get_connection().await;
        let mut board_repo: Repository<BoardAggregate, BoardEvent> =
            functions::board_repository_helper(connection.clone()).await;
        //* values for comparison, fetch
        let id: String;
        let existing_content: String;

        '_transaction_block: {
            connection.write().await.begin().await.unwrap();

            let mut board_aggregate = functions::board_create_helper(BoardState::Unpublished);
            existing_content = board_aggregate.board.content.clone();

            id = board_repo.add(&mut board_aggregate).await.unwrap();

            connection.write().await.commit().await.unwrap();
        }

        '_transaction_block2: {
            connection.write().await.begin().await.unwrap();

            let mut initial_board_aggregate = board_repo.get(&id).await.unwrap();
            let initial_board = &mut initial_board_aggregate.board;

            '_test_block: {
                assert_eq!(initial_board.state(), "Unpublished");
                assert_eq!(initial_board.version, 0);
            }
            let new_author = Uuid::new_v4();
            initial_board.author = new_author;
            initial_board.content = "Something else".to_string();
            board_repo.update(initial_board_aggregate).await.unwrap();

            connection.write().await.commit().await.unwrap();

            '_test_block: {
                let board_aggregate = board_repo.get(&id).await.unwrap();
                let updated_board = board_aggregate.board;

                assert_eq!(updated_board.author, new_author);
                assert_ne!(updated_board.content, existing_content);
                assert_eq!(updated_board.version, 1);
            }
        }

        functions::tear_down().await;
    }

    #[tokio::test]
    async fn test_create_comment() {
        let connection = functions::get_connection().await;
        let mut board_repo: Repository<BoardAggregate, BoardEvent> =
            functions::board_repository_helper(connection.clone()).await;
        //* values for comparison, fetch
        let id: String;
        let mut board_aggregate: BoardAggregate;

        '_tranasction_block: {
            connection.write().await.begin().await.unwrap();
            board_aggregate = functions::board_create_helper(BoardState::Unpublished);
            id = board_repo.add(&mut board_aggregate).await.unwrap();
            connection.write().await.commit().await.unwrap();
        }

        '_transaction_block2: {
            let comment = functions::comment_create_helper(&id);

            let mut board_builder = BoardAggregate::builder();
            board_builder = board_builder
                .take_board(board_aggregate.board)
                .take_comments(vec![comment]);

            connection.write().await.begin().await.unwrap();
            board_repo.update(board_builder.build()).await.unwrap();
            connection.write().await.commit().await.unwrap();
        }

        '_test_block: {
            let board_aggregate = board_repo.get(&id).await.unwrap();

            assert_eq!(board_aggregate.board.version, 1);
            assert_eq!(board_aggregate.comments.len(), 1);
        }
    }
}
