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
        adapters::{
            database::{AtomicContextManager, ContextManager},
            outbox::Outbox,
            repositories::TRepository,
        },
        domain::board::entity::BoardState,
        services::unit_of_work::UnitOfWork,
    };

    async fn outbox_setup(connection: AtomicContextManager) {
        let cmd = CreateBoard {
            author: Uuid::new_v4(),
            title: "Title!".to_string(),
            content: "Content".to_string(),
            state: BoardState::Published,
        };

        let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(connection.clone());
        match ServiceHandler::create_board(cmd, connection.clone()).await {
            Err(err) => '_fail_case: {
                panic!("Service Handling Failed! {}", err)
            }
            Ok(response) => '_test: {
                let id: String = response.try_into().unwrap();

                if let Err(err) = uow.repository.get(&id).await {
                    panic!("Fetching newly created object failed! : {}", err);
                };
            }
        }
    }

    #[tokio::test]
    async fn test_create_board_leaves_outbox() {
        run_test(async {
            let connection = ContextManager::new().await.unwrap();
            outbox_setup(connection.clone()).await;

            '_test_case: {
                match Outbox::get(connection.clone()).await {
                    Err(err) => {
                        eprintln!("{}", err);
                        panic!("Test Failed! Outbox Not Stored!")
                    }
                    Ok(val) => {
                        println!("{:?}", val);
                        println!("Outbox stored successfully!")
                    }
                }
            }
        })
        .await
    }

    #[tokio::test]
    async fn test_convert_event() {
        run_test(async {
            let connection = ContextManager::new().await.unwrap();
            outbox_setup(connection.clone()).await;

            '_test_case: {
                let vec_of_outbox = Outbox::get(connection.clone()).await.unwrap();

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

    #[tokio::test]
    async fn test_outbox_event_handled_by_messagebus() {
        run_test(async {
            let connection = ContextManager::new().await.unwrap();
            outbox_setup(connection.clone()).await;

            '_test_case: {
                let bus = Boostrap::message_bus().await;

                for e in Outbox::get(connection.clone()).await.unwrap() {
                    //TODO Messagebus for outbox?
                    match bus.handle(e).await {
                        Ok(_var) => {
                            println!("Success!")
                        }
                        Err(_) => panic!("Failed!"),
                    }
                }

                let boxes = Outbox::get(connection.clone()).await.unwrap();
                assert!(boxes.is_empty());
            }
        })
        .await
    }
}
