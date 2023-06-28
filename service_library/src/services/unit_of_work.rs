use std::{collections::VecDeque, mem, sync::Arc};

use tokio::sync::Mutex;

use crate::adapters::repositories::{Repository, TRepository};
use crate::domain::Message;
use crate::{
    adapters::{database::AtomicConnection, outbox::Outbox},
    domain::{auth::AuthAggregate, board::BoardAggregate},
    utils::ApplicationResult,
};

pub(crate) type AtomicUnitOfWork = Arc<Mutex<UnitOfWork>>;
pub struct UnitOfWork {
    pub connection: AtomicConnection,
    pub boards: Repository<BoardAggregate>,
    pub auths: Repository<AuthAggregate>,
}

impl UnitOfWork {
    pub fn new(connection: AtomicConnection) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            connection: connection.clone(),
            boards: Repository::new(connection.clone()),
            auths: Repository::new(connection),
        }))
    }

    pub async fn begin(&mut self) {
        if let Err(err) = self.boards.connection().write().await.begin().await {
            eprintln!("Transaction Error! : {}", err);
        }
    }

    pub async fn commit(&mut self) -> ApplicationResult<()> {
        self._save_outboxes(self.connection.clone()).await?;
        self.connection.write().await.commit().await?;
        Ok(())
    }

    pub async fn rollback(&mut self) -> ApplicationResult<()> {
        self.boards.connection().write().await.rollback().await?;
        Ok(())
    }
    pub async fn _save_outboxes(&self, connection: AtomicConnection) -> ApplicationResult<()> {
        Outbox::add(
            connection,
            self.boards
                ._collect_outbox()
                .chain(self.auths._collect_outbox())
                .collect::<Vec<_>>(),
        )
        .await?;

        Ok(())
    }
    pub fn _collect_events(&mut self) -> VecDeque<Box<dyn Message>> {
        let mut events: VecDeque<Box<dyn Message>> =
            VecDeque::with_capacity(self.boards.events.len() + self.auths.events.len());
        mem::take(&mut self.boards.events)
            .into_iter()
            .for_each(|e| events.push_back(e));

        mem::take(&mut self.auths.events)
            .into_iter()
            .for_each(|e| events.push_back(e));
        events
    }
}

//TODO Using UOW, transaction handling
#[cfg(test)]
mod test_unit_of_work {

    use uuid::Uuid;

    use crate::adapters::database::Connection;
    use crate::adapters::repositories::TRepository;
    use crate::domain::board::commands::CreateBoard;
    use crate::domain::board::{
        entity::{Board, BoardState},
        BoardAggregate,
    };
    use crate::domain::builder::{Buildable, Builder};
    use crate::services::unit_of_work::UnitOfWork;
    use crate::utils::test_components::components::*;

    #[tokio::test]
    async fn test_unit_of_work() {
        run_test(async {
            let connection = Connection::new().await.unwrap();
            let uow = UnitOfWork::new(connection);

            '_transaction_block: {
                let builder = BoardAggregate::builder();
                let boardaggregate = builder
                    .take_board(Board::new(
                        Uuid::new_v4(),
                        "Title!",
                        "Content!",
                        BoardState::Published,
                    ))
                    .build();
                let id: String = boardaggregate.board.id.to_string();
                let mut uow = uow.lock().await;
                uow.begin().await;
                uow.boards.add(boardaggregate).await.unwrap();
                uow.commit().await.unwrap();

                '_test_block: {
                    if let Err(err) = uow.boards.get(&id).await {
                        panic!("Fetch Error!:{}", err)
                    };
                }
            }

            '_transaction_block: {
                let builder = BoardAggregate::builder();
                let boardaggregate = builder
                    .take_board(Board::new(
                        Uuid::new_v4(),
                        "Title!",
                        "Content!",
                        BoardState::Published,
                    ))
                    .build();
                let id: String = boardaggregate.board.id.to_string();
                let mut uow = uow.lock().await;
                uow.begin().await;
                uow.boards.add(boardaggregate).await.unwrap();
                uow.rollback().await.unwrap();

                '_test_block: {
                    if let Ok(_val) = uow.boards.get(&id).await {
                        panic!("Shouldn't be able to fetch after rollback!!")
                    };
                }
            }
        })
        .await
    }

    #[tokio::test]
    async fn test_unit_of_work_event_collection() {
        run_test(async {
            let connection = Connection::new().await.unwrap();
            let uow = UnitOfWork::new(connection);
            '_transaction_block: {
                let builder = BoardAggregate::builder();
                let mut boardaggregate = builder.build();

                // The following method on aggregate raises an event
                boardaggregate.create_board(CreateBoard {
                    author: Uuid::new_v4(),
                    title: "Title!".into(),
                    content: "Content".into(),
                    state: BoardState::Published,
                });
                let id: String = boardaggregate.board.id.to_string();

                let mut uow = uow.lock().await;
                uow.begin().await;
                uow.boards.add(boardaggregate).await.unwrap();
                uow.commit().await.unwrap();

                '_test_block: {
                    if let Err(err) = uow.boards.get(&id).await {
                        panic!("Fetch Error!:{}", err)
                    };
                    let vec = uow._collect_events();
                    assert_eq!(vec.len(), 1);

                    // When you try it again, it should be empty
                    let vec = uow._collect_events();
                    assert_eq!(vec.len(), 0)
                }
            }
        })
        .await;
    }
}
