mod helpers;
#[cfg(test)]
pub mod service_tests {

    use std::str::FromStr;

    use crate::helpers::functions::*;

    use service_library::adapters::database::ContextManager;
    use service_library::adapters::repositories::{Repository, TRepository};

    use service_library::domain::board::commands::{AddComment, CreateBoard, EditBoard};
    use service_library::domain::board::entity::BoardState;
    use service_library::domain::board::BoardAggregate;
    use service_library::domain::Message;

    use service_library::services::handlers::ServiceHandler;
    use service_library::services::unit_of_work::UnitOfWork;
    use tokio::sync::mpsc;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_board() {
        run_test(async {
            let (sx, _) = mpsc::unbounded_channel::<Box<dyn Message>>();
            let context_manager = ContextManager::new(sx).await;
            let pool = context_manager.read().await.pool;
            let cmd = CreateBoard {
                author: Uuid::new_v4(),
                title: "Title!".to_string(),
                content: "Content".to_string(),
                state: BoardState::Published,
            };
            let (sx, _) = mpsc::unbounded_channel::<Box<dyn Message>>();

            let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                context_manager.read().await.pool,
            );
            match ServiceHandler::create_board(cmd, context_manager.clone()).await {
                Err(err) => '_fail_case: {
                    panic!("Service Handling Failed! {}", err)
                }
                Ok(id) => '_test: {
                    let id: String = id.try_into().unwrap();
                    if let Err(err) = uow.repository.get(&id).await {
                        panic!("Fetching newly created object failed! : {}", err);
                    };
                }
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_edit_board() {
        run_test(async {
            let (sx, _) = mpsc::unbounded_channel::<Box<dyn Message>>();
            let context_manager = ContextManager::new(sx).await;
            let pool = context_manager.read().await.pool;
            let id: String;
            let (sx, _) = mpsc::unbounded_channel::<Box<dyn Message>>();
            let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                context_manager.read().await.pool,
            );
            '_preparation_block: {
                let mut board_repo = board_repository_helper(uow.executor.clone()).await;

                uow.begin().await.unwrap();

                let mut board_aggregate = board_create_helper(BoardState::Published);
                id = board_repo.add(&mut board_aggregate).await.unwrap();
                assert_eq!(board_aggregate.board.id.to_string(), id);
                uow.commit().await.unwrap();
            }

            '_test_block: {
                let cmd = EditBoard {
                    id: Uuid::from_str(&id).unwrap(),
                    title: None,
                    content: Some("Changed to this".to_string()),
                    state: None,
                };

                let id = cmd.id.clone().to_string();

                match ServiceHandler::edit_board(cmd, context_manager.clone()).await {
                    Err(err) => '_fail_case: {
                        panic!("Service Handling Failed! {}", err)
                    }
                    Ok(_res) => {
                        let uow =
                            UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(pool);
                        if let Ok(board_aggregate) = uow.repository.get(&id).await {
                            assert_eq!(
                                board_aggregate.board.content,
                                "Changed to this".to_string()
                            );
                        };
                    }
                }
            }
        })
        .await;
    }

    #[tokio::test]
    async fn test_add_comment() {
        run_test(async {
            let (sx, _) = mpsc::unbounded_channel::<Box<dyn Message>>();
            let context_manager = ContextManager::new(sx).await;

            let id: String;
            let (sx, _) = mpsc::unbounded_channel::<Box<dyn Message>>();
            let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                context_manager.read().await.pool,
            );
            '_preparation_block: {
                let mut board_repo = board_repository_helper(uow.executor.clone()).await;

                uow.begin().await.unwrap();

                let mut board_aggregate = board_create_helper(BoardState::Published);
                id = board_repo.add(&mut board_aggregate).await.unwrap();
                assert_eq!(board_aggregate.board.id.to_string(), id);
                uow.commit().await.unwrap();
            }

            '_test_block: {
                let cmd = AddComment {
                    board_id: Uuid::from_str(&id).unwrap(),
                    author: Uuid::new_v4(),
                    content: "What a beautiful day!".to_string(),
                };
                ServiceHandler::add_comment(cmd, context_manager.clone())
                    .await
                    .unwrap();
                let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                    context_manager.read().await.pool,
                );
                if let Ok(board_aggregate) = uow.repository.get(&id).await {
                    assert_eq!(board_aggregate.comments.len(), 1);
                };
            }
        })
        .await;
    }
}
