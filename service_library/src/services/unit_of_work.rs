use std::collections::VecDeque;
use std::marker::PhantomData;

use std::sync::Arc;

use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::adapters::database::Executor;
use crate::adapters::repositories::TRepository;
use crate::domain::{Aggregate, Message};
use crate::utils::ApplicationError;
use crate::{adapters::outbox::Outbox, utils::ApplicationResult};

pub struct UnitOfWork<R, A>
where
    R: TRepository<A>,
    A: Aggregate + 'static,
{
    pub executor: Arc<RwLock<Executor>>,
    pub repository: R,
    pub _aggregate: PhantomData<A>,
}

impl<R, A> UnitOfWork<R, A>
where
    R: TRepository<A>,
    A: Aggregate,
{
    pub fn new(pool: &'static PgPool) -> Self {
        let executor = Arc::new(RwLock::new(Executor::new(pool)));
        Self {
            repository: R::new(executor.clone()),
            executor,
            _aggregate: PhantomData::<A>,
        }
    }

    pub async fn begin(&mut self) -> Result<(), ApplicationError> {
        // TODO Need to be simplified
        let mut executor = self.executor.write().await;

        executor.begin().await?;
        Ok(())
    }

    pub async fn commit(mut self) -> ApplicationResult<()> {
        // To drop uow itself!
        self._save_outboxes(self.executor.clone()).await?;
        self._collect_events().await;

        self._commit().await
    }
    async fn _commit(&mut self) -> ApplicationResult<()> {
        let mut executor = self.executor.write().await;
        executor.commit().await
    }

    pub async fn rollback(self) -> ApplicationResult<()> {
        let mut executor = self.executor.write().await;
        executor.rollback().await
    }

    pub async fn _save_outboxes(&self, executor: Arc<RwLock<Executor>>) -> ApplicationResult<()> {
        Outbox::add(
            executor,
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

        //TODO Send event
        // self.connection.write().await.events = events;
    }
}

//TODO Using UOW, transaction handling
#[cfg(test)]
mod test_unit_of_work {

    use tokio::sync::mpsc;
    use uuid::Uuid;

    use crate::adapters::database::connection_pool;

    use crate::adapters::repositories::{Repository, TRepository};
    use crate::domain::board::commands::CreateBoard;
    use crate::domain::board::{
        entity::{Board, BoardState},
        BoardAggregate,
    };
    use crate::domain::builder::{Buildable, Builder};
    use crate::domain::Message;
    use crate::services::unit_of_work::UnitOfWork;
    use crate::utils::test_components::components::*;

    #[tokio::test]
    async fn test_unit_of_work() {
        run_test(async {
            let pool = connection_pool().await;

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

                let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(pool);
                uow.begin().await.unwrap();
                uow.repository.add(&mut boardaggregate).await.unwrap();
                uow.commit().await.unwrap();

                '_test_block: {
                    let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(pool);
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
                let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(pool);
                uow.begin().await.unwrap();
                uow.repository.add(&mut boardaggregate).await.unwrap();
                uow.rollback().await.unwrap();

                '_test_block: {
                    let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(pool);
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
            let pool = connection_pool().await;
            // TODO Subject to deletion
            let (sx, mut rx) = mpsc::unbounded_channel::<Box<dyn Message>>();

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
                let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(pool);
                uow.begin().await.unwrap();
                uow.repository.add(&mut boardaggregate).await.unwrap();
                uow.commit().await.unwrap();

                '_test_block: {
                    let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(pool);
                    if let Err(err) = uow.repository.get(&id).await {
                        panic!("Fetch Error!:{}", err)
                    };
                    let mut count = 0;
                    while let Some(vec_msg) = rx.recv().await {
                        count += 1
                    }
                    assert_eq!(count, 1)
                }
            }
        })
        .await;
    }
}
