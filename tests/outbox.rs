mod helpers;

#[cfg(test)]
mod test_outbox {
    use crate::helpers::functions::*;
    use core::panic;
    use service_library::adapters::repositories::Repository;
    use service_library::bootstrap::Boostrap;
    use service_library::domain::board::commands::CreateBoard;
    use service_library::domain::board::events::BoardCreated;
    use service_library::domain::board::BoardAggregate;
    use service_library::services::handlers::ServiceHandler;

    use uuid::Uuid;

    use service_library::{
        adapters::{database::ContextManager, outbox::Outbox, repositories::TRepository},
        domain::board::entity::BoardState,
        services::unit_of_work::UnitOfWork,
    };

    async fn outbox_setup() {
        let cmd = CreateBoard {
            author: Uuid::new_v4(),
            title: "Title!".to_string(),
            content: "Content".to_string(),
            state: BoardState::Published,
        };

        // ! The Following receiver must exist
        let (context_manager, mut _receiver) = ContextManager::new().await;

        match ServiceHandler::create_board(cmd, context_manager.clone()).await {
            Err(err) => '_fail_case: {
                panic!("Service Handling Failed! {}", err)
            }
            Ok(response) => '_test: {
                let id: String = response.try_into().unwrap();
                let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                    context_manager.clone(),
                )
                .await;
                if let Err(err) = uow.repository.get(&id).await {
                    panic!("Fetching newly created object failed! : {}", err);
                };
            }
        }
    }

    #[tokio::test]
    async fn test_create_board_leaves_outbox() {
        run_test(async {
            let (context_manager, _) = ContextManager::new().await;
            outbox_setup().await;

            '_test_case: {
                match Outbox::get(context_manager.read().await.executor()).await {
                    Err(err) => {
                        eprintln!("{}", err);
                        panic!("Test Failed! Outbox Not Stored!")
                    }
                    Ok(val) => {
                        println!("{:?}", val);
                        println!("Outbox stored successfully!")
                    }
                };
            }
        })
        .await
    }

    #[tokio::test]
    async fn test_convert_event() {
        run_test(async {
            let (context_manager, _) = ContextManager::new().await;
            outbox_setup().await;

            '_test_case: {
                let vec_of_outbox = Outbox::get(context_manager.read().await.executor())
                    .await
                    .unwrap();

                assert_eq!(vec_of_outbox.len(), 1);
                let event = vec_of_outbox.get(0).unwrap().convert_event();
                assert!(event.externally_notifiable());

                let converted = serde_json::from_str(&event.state()).unwrap();
                match converted {
                    BoardCreated { .. } => {
                        println!("Success!")
                    }
                    _ => {
                        panic!("Failed!")
                    }
                };
            }
        })
        .await
    }

    #[tokio::main]
    #[test]
    async fn test_outbox_event_handled_by_messagebus() {
        run_test(async {
            let (context_manager, _rv) = ContextManager::new().await;

            let executor = context_manager.read().await.executor();
            drop(context_manager);

            outbox_setup().await;

            '_test_case: {
                let bus = Boostrap::message_bus().await;

                for e in Outbox::get(executor.clone()).await.unwrap() {
                    //TODO Messagebus for outbox?
                    match bus.handle(e).await {
                        Ok(_var) => {
                            println!("Success!")
                        }
                        Err(_) => panic!("Failed!"),
                    }
                }

                let boxes = Outbox::get(executor).await.unwrap();
                assert!(boxes.is_empty());
            }
        })
        .await
    }
}
