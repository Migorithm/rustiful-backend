use std::collections::VecDeque;
use std::marker::PhantomData;

use crate::adapters::repositories::TRepository;
use crate::domain::{Aggregate, Message};
use crate::{
    adapters::{database::AtomicConnection, outbox::Outbox},
    utils::ApplicationResult,
};

pub struct UnitOfWork<R, A>
where
    R: TRepository<A>,
    A: Aggregate + 'static,
{
    pub connection: AtomicConnection,
    // pub boards: Repository<BoardAggregate>,
    // pub auths: Repository<AuthAggregate>,
    pub repository: R,
    pub _aggregate: PhantomData<A>,
}

impl<R, A> UnitOfWork<R, A>
where
    R: TRepository<A>,
    A: Aggregate,
{
    pub fn new(connection: AtomicConnection) -> Self {
        Self {
            connection: connection.clone(),
            repository: R::new(connection),
            _aggregate: PhantomData::<A>,
        }
    }

    pub async fn begin(&mut self) {
        if let Err(err) = self.connection.write().await.begin().await {
            eprintln!("Transaction Error! : {}", err);
        }
    }

    pub async fn commit(mut self) -> ApplicationResult<()> {
        // To drop uow itself!
        self._save_outboxes(self.connection.clone()).await?;
        self._collect_events().await;
        self.connection.write().await.commit().await?;

        Ok(())
    }

    pub async fn rollback(self) -> ApplicationResult<()> {
        self.connection.write().await.rollback().await?;
        Ok(())
    }
    pub async fn _save_outboxes(&self, connection: AtomicConnection) -> ApplicationResult<()> {
        Outbox::add(
            connection,
            self.repository._collect_outbox().collect::<Vec<_>>(),
        )
        .await?;

        Ok(())
    }
    pub async fn _collect_events(&mut self) {
        let mut events: VecDeque<Box<dyn Message>> =
            VecDeque::with_capacity(self.repository.get_events().len());
        self.repository
            .get_events()
            .iter()
            .for_each(|e| events.push_back(e.message_clone()));

        self.connection.write().await.events = events;
    }
}

//TODO Using UOW, transaction handling
#[cfg(test)]
mod test_unit_of_work {

    use uuid::Uuid;

    use crate::adapters::database::Connection;

    use crate::adapters::repositories::{Repository, TRepository};
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

            '_transaction_block: {
                let builder = BoardAggregate::builder();
                let mut boardaggregate = builder
                    .take_board(Board::new(
                        Uuid::new_v4(),
                        "Title!",
                        "Content!",
                        BoardState::Published,
                    ))
                    .build();
                let id: String = boardaggregate.board.id.to_string();
                let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                    connection.clone(),
                );
                uow.begin().await;
                uow.repository.add(&mut boardaggregate).await.unwrap();
                uow.commit().await.unwrap();

                '_test_block: {
                    let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                        connection.clone(),
                    );
                    if let Err(err) = uow.repository.get(&id).await {
                        panic!("Fetch Error!:{}", err)
                    };
                }
            }

            '_transaction_block: {
                let builder = BoardAggregate::builder();
                let mut boardaggregate = builder
                    .take_board(Board::new(
                        Uuid::new_v4(),
                        "Title!",
                        "Content!",
                        BoardState::Published,
                    ))
                    .build();
                let id: String = boardaggregate.board.id.to_string();
                let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                    connection.clone(),
                );
                uow.begin().await;
                uow.repository.add(&mut boardaggregate).await.unwrap();
                uow.rollback().await.unwrap();

                '_test_block: {
                    let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                        connection.clone(),
                    );
                    if let Ok(_val) = uow.repository.get(&id).await {
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
                let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                    connection.clone(),
                );
                uow.begin().await;
                uow.repository.add(&mut boardaggregate).await.unwrap();
                uow.commit().await.unwrap();

                '_test_block: {
                    let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                        connection.clone(),
                    );
                    if let Err(err) = uow.repository.get(&id).await {
                        panic!("Fetch Error!:{}", err)
                    };
                    let vec = &connection.read().await.events;
                    assert_eq!(vec.len(), 1);
                }
            }
        })
        .await;
    }
}
