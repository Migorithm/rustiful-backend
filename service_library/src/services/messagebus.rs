use crate::{
    adapters::database::AtomicConnection,
    domain::{
        auth::events::AuthEvent,
        board::events::BoardEvent,
        commands::{ApplicationCommand, Command},
        AnyTrait,
    },
    utils::{ApplicationError, ApplicationResult},
};

use std::{any::Any, collections::VecDeque, marker::PhantomData, sync::Arc};
use tokio::sync::Mutex;

use super::{
    handlers::{self, CommandHandler, Future},
    unit_of_work::UnitOfWork,
};

#[derive(Clone)]
pub struct MessageBus<C = ApplicationCommand>
where
    C: Command,
{
    _phantom: PhantomData<C>,
    #[cfg(test)]
    pub book_keeper: i32,
}

impl Default for MessageBus {
    fn default() -> Self {
        Self {
            _phantom: Default::default(),
            #[cfg(test)]
            book_keeper: 0,
        }
    }
}

impl<C: Command + AnyTrait> MessageBus<C> {
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData::<C>,

            #[cfg(test)]
            book_keeper: 0,
        }
    }

    pub async fn handle(
        &mut self,
        message: C,
        connection: AtomicConnection,
    ) -> ApplicationResult<VecDeque<C::Response>> {
        //TODO event generator
        let uow = UnitOfWork::new(connection.clone());

        let mut queue = VecDeque::from([message.as_any()]);
        let mut res_queue = Mutex::new(VecDeque::new());

        while let Some(msg) = queue.pop_front() {
            // * Logging!

            if msg.is::<C>() {
                let cmd: C = *msg.downcast::<C>().unwrap();
                let res = self.handle_command(cmd, uow.clone()).await?;
                res_queue.get_mut().push_back(res);

                #[cfg(test)]
                {
                    self.book_keeper += 1;
                };
            } else {
                match self.handle_event(msg, uow.clone()).await {
                    Err(ApplicationError::StopSentinel) => {
                        eprintln!("Stop Sentinel Reached!");
                        break;
                    }
                    Err(err) => {
                        eprintln!("Error Occurred While Handling Event! Error:{}", err);
                    }
                    Ok(_) => {
                        println!("Event Handling Succeeded!")
                    }
                }
            }

            for new_event in uow.clone().lock().await._collect_events() {
                queue.push_back(new_event);
            }
        }
        drop(uow);
        Ok(res_queue.into_inner())
    }

    async fn handle_command(
        &mut self,
        command: C,
        uow: Arc<Mutex<UnitOfWork>>,
    ) -> ApplicationResult<C::Response> {
        let handler: CommandHandler<C, C::Response> = command.execute();

        let fut: Future<C::Response> = handler(command, uow);
        Ok(fut.await.expect("Error Occurred While Handling Command!"))
    }

    async fn handle_event(
        &mut self,
        msg: Box<dyn Any + Send + Sync>,
        uow: Arc<Mutex<UnitOfWork>>,
    ) -> ApplicationResult<()> {
        if msg.is::<BoardEvent>() {
            let event: BoardEvent = *msg.downcast::<BoardEvent>().unwrap();
            match event {
                BoardEvent::Created { .. } => {
                    for handler in handlers::BOARD_CREATED_EVENT_HANDLERS.iter() {
                        handler(event.clone(), uow.clone()).await?;
                    }
                }
                BoardEvent::Updated { .. } => {
                    for handler in handlers::BOARD_UPDATED_EVENT_HANDLERS.iter() {
                        handler(event.clone(), uow.clone()).await?;
                    }
                }
                BoardEvent::CommentAdded { .. } => {
                    for handler in handlers::COMMENT_ADDED_EVENT_HANDLERS.iter() {
                        handler(event.clone(), uow.clone()).await?;
                    }
                }
            };
            #[cfg(test)]
            {
                self.book_keeper += 1
            }
        } else if msg.is::<AuthEvent>() {
            let event: AuthEvent = *msg.downcast::<AuthEvent>().unwrap();
            match event {
                AuthEvent::Created { .. } => {
                    for handler in handlers::ACCOUNT_CREATED_EVENT_HANDLERS.iter() {
                        handler(event.clone(), uow.clone()).await?;
                    }
                }
                AuthEvent::Updated { .. } => {
                    for handler in handlers::ACCOUNT_UPDATED_EVENT_HANDLERS.iter() {
                        handler(event.clone(), uow.clone()).await?;
                    }
                }
            };
            #[cfg(test)]
            {
                self.book_keeper += 1
            }
        } else {
            Err(ApplicationError::InExecutableEvent)?
        }

        drop(uow);
        Ok(())
    }
}

#[cfg(test)]
pub mod test_messagebus {

    use std::marker::PhantomData;

    use crate::adapters::database::Connection;
    use crate::domain::commands::ApplicationCommand;

    use crate::services::messagebus::MessageBus;
    use crate::utils::test_components::components::*;

    use uuid::Uuid;

    #[test]
    fn test_message_default() {
        let ms = MessageBus::default();
        assert_eq!(ms._phantom, PhantomData::<ApplicationCommand>)
    }

    #[tokio::test]
    async fn test_message_bus_command_handling() {
        run_test(|| {
            Box::pin(async {
                let connection = Connection::new().await.unwrap();
                let mut ms = MessageBus::default();
                let cmd = ApplicationCommand::CreateBoard {
                    author: Uuid::new_v4(),
                    title: "TestTitle".into(),
                    content: "TestContent".into(),
                    state: Default::default(),
                };
                match ms.handle(cmd, connection).await {
                    Ok(mut res_queue) => {
                        let res = res_queue
                            .pop_front()
                            .expect("There Must Be A Result String!");
                        println!("{:?}", res)
                    }
                    Err(err) => {
                        eprintln!("{}", err);
                        panic!("Test Failed!")
                    }
                };

                assert_eq!(ms.book_keeper, 2);
            })
        })
        .await;
    }
}
