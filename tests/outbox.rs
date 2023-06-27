mod helpers;

#[cfg(test)]
mod test_outbox {
    use crate::helpers::functions::*;
    use core::panic;
    use service_library::domain::board::commands::CreateBoard;
    use uuid::Uuid;

    use service_library::domain::board::events::BoardEvent;

    use service_library::services::messagebus::MessageBus;

    use service_library::{
        adapters::{
            database::{AtomicConnection, Connection},
            outbox::Outbox,
            repositories::TRepository,
        },
        domain::{board::entity::BoardState, commands::Command},
        services::unit_of_work::UnitOfWork,
    };

    async fn outbox_setup(connection: AtomicConnection) {
        let cmd = CreateBoard {
            author: Uuid::new_v4(),
            title: "Title!".to_string(),
            content: "Content".to_string(),
            state: BoardState::Published,
        };

        let uow = UnitOfWork::new(connection.clone());
        match cmd.handle(uow.clone()).await {
            Err(err) => '_fail_case: {
                panic!("Service Handling Failed! {}", err)
            }
            Ok(response) => '_test: {
                let uow = UnitOfWork::new(connection);

                if let Err(err) = uow.lock().await.boards.get(&response).await {
                    panic!("Fetching newly created object failed! : {}", err);
                };
            }
        }
    }

    #[tokio::test]
    async fn test_create_board_leaves_outbox() {
        run_test(async {
            let connection = Connection::new().await.unwrap();
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
            let connection = Connection::new().await.unwrap();
            outbox_setup(connection.clone()).await;

            '_test_case: {
                let vec_of_outbox = Outbox::get(connection.clone()).await.unwrap();

                assert_eq!(vec_of_outbox.len(), 1);
                let event = vec_of_outbox.get(0).unwrap().convert_event();
                assert!(event.is::<BoardEvent>());

                let converted = *event.downcast::<BoardEvent>().unwrap();
                match converted {
                    BoardEvent::Created { .. } => {
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
            let connection = Connection::new().await.unwrap();
            outbox_setup(connection.clone()).await;

            '_test_case: {
                let mut bus = MessageBus::<Outbox>::new();

                for e in Outbox::get(connection.clone()).await.unwrap() {
                    match bus.handle(e, connection.clone()).await {
                        Ok(_var) => {
                            println!("Success!")
                        }
                        Err(_) => panic!("Failed!"),
                    }
                }

                // TODO where does the processed tag get modifeid?
                let boxes = Outbox::get(connection.clone()).await.unwrap();
                assert!(boxes.is_empty());
            }
        })
        .await
    }
}
