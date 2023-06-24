mod helpers;
#[cfg(test)]
pub mod service_tests {

    use std::str::FromStr;

    use crate::helpers::functions::*;
    use service_library::adapters::database::Connection;
    use service_library::adapters::repository::TRepository;

    use service_library::domain::board::entity::BoardState;
    use service_library::domain::commands::ApplicationCommand;
    use service_library::services::handlers::{Handler, ServiceHandler};
    use service_library::services::unit_of_work::UnitOfWork;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_board() {
        let cmd = ApplicationCommand::CreateBoard {
            author: Uuid::new_v4(),
            title: "Title!".to_string(),
            content: "Content".to_string(),
            state: BoardState::Published,
        };

        let connection = Connection::new().await.unwrap();
        let uow = UnitOfWork::new(connection.clone());
        match ServiceHandler::execute(cmd, uow.clone()).await {
            Err(err) => '_fail_case: {
                panic!("Service Handling Failed! {}", err)
            }
            Ok(id) => '_test: {
                let uow = UnitOfWork::new(connection.clone());
                if let Err(err) = uow.lock().await.boards.get(&id.to_str()).await {
                    panic!("Fetching newly created object failed! : {}", err);
                };
            }
        }
        tear_down().await;
    }

    #[tokio::test]
    async fn test_edit_board() {
        let connection = Connection::new().await.unwrap();
        let id: String;
        let uow = UnitOfWork::new(connection.clone());
        '_preparation_block: {
            let mut board_repo = board_repository_helper(connection.clone()).await;

            connection.write().await.begin().await.unwrap();

            let mut board_aggregate = board_create_helper(BoardState::Published);
            id = board_repo.add(&mut board_aggregate).await.unwrap();
            assert_eq!(board_aggregate.board.id.to_string(), id);
            connection.write().await.commit().await.unwrap();
        }

        '_test_block: {
            let cmd = ApplicationCommand::EditBoard {
                id: Uuid::from_str(&id).unwrap(),
                title: None,
                content: Some("Changed to this".to_string()),
                state: None,
            };

            match ServiceHandler::execute(cmd, uow.clone()).await {
                Err(err) => '_fail_case: {
                    panic!("Service Handling Failed! {}", err)
                }
                Ok(id) => {
                    let uow = UnitOfWork::new(connection.clone());
                    if let Ok(board_aggregate) = uow.lock().await.boards.get(&id.to_str()).await {
                        assert_eq!(board_aggregate.board.content, "Changed to this".to_string());
                    };
                }
            }
        }
        tear_down().await;
    }

    #[tokio::test]
    async fn test_add_comment() {
        let connection = Connection::new().await.unwrap();
        let id: String;
        let uow = UnitOfWork::new(connection.clone());
        '_preparation_block: {
            let mut board_repo = board_repository_helper(connection.clone()).await;

            connection.write().await.begin().await.unwrap();

            let mut board_aggregate = board_create_helper(BoardState::Published);
            id = board_repo.add(&mut board_aggregate).await.unwrap();
            assert_eq!(board_aggregate.board.id.to_string(), id);
            connection.write().await.commit().await.unwrap();
        }

        '_test_block: {
            let cmd = ApplicationCommand::AddComment {
                board_id: Uuid::from_str(&id).unwrap(),
                author: Uuid::new_v4(),
                content: "What a beautiful day!".to_string(),
            };
            ServiceHandler::execute(cmd, uow.clone()).await.unwrap();

            let uow = UnitOfWork::new(connection.clone());
            if let Ok(board_aggregate) = uow.lock().await.boards.get(&id).await {
                assert_eq!(board_aggregate.comments.len(), 1);
            };
        }
        tear_down().await;
    }
}
