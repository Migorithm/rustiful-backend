use std::marker::PhantomData;

use std::sync::Arc;

use tokio::sync::RwLock;

use crate::adapters::database::{AtomicContextManager, Executor};
use crate::adapters::repositories::TRepository;
use crate::domain::Aggregate;
use crate::utils::ApplicationError;
use crate::{adapters::outbox::Outbox, utils::ApplicationResult};

pub struct UnitOfWork<R, A>
where
    R: TRepository<A>,
    A: Aggregate + 'static,
{
    executor: Arc<RwLock<Executor>>,
    context: AtomicContextManager,
    pub repository: R,
    _aggregate: PhantomData<A>,
}
impl<R, A> UnitOfWork<R, A>
where
    R: TRepository<A>,
    A: Aggregate,
{
    pub async fn new(context: AtomicContextManager) -> Self {
        let executor = context.read().await.executor();
        Self {
            repository: R::new(executor.clone()),
            context,
            executor,
            _aggregate: PhantomData::<A>,
        }
    }

    pub async fn begin(&mut self) -> Result<(), ApplicationError> {
        // TODO Need to be simplified
        let mut executor = self.executor.write().await;
        executor.begin().await
    }

    pub fn executor(&self) -> Arc<RwLock<Executor>> {
        self.executor.clone()
    }

    pub async fn commit(mut self) -> ApplicationResult<()> {
        // To drop uow itself!

        self._commit_hook().await?;

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

    /// commit_hook is invoked right before the calling for commit
    /// which sorts out and processes outboxes and internally processable events.
    pub async fn _commit_hook(&mut self) -> ApplicationResult<()> {
        let event_sender = &mut self.context.write().await.sender;
        let mut outboxes = vec![];

        for e in self.repository.get_events() {
            if e.externally_notifiable() {
                outboxes.push(e.outbox());
            };
            if e.internally_notifiable() {
                event_sender
                    .send(e.message_clone())
                    .await
                    .expect("Event Collecting failed!")
            }
        }
        Outbox::add(self.executor(), outboxes).await
    }
}

//TODO Using UOW, transaction handling
#[cfg(test)]
mod test_unit_of_work {

    use uuid::Uuid;

    use crate::adapters::database::ContextManager;

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
            let (ctx_manager, _) = ContextManager::new().await;

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
                    ctx_manager.clone(),
                )
                .await;
                uow.begin().await.unwrap();
                uow.repository.add(&mut boardaggregate).await.unwrap();
                uow.commit().await.unwrap();

                '_test_block: {
                    let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                        ctx_manager.clone(),
                    )
                    .await;
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
                    ctx_manager.clone(),
                )
                .await;
                uow.begin().await.unwrap();
                uow.repository.add(&mut boardaggregate).await.unwrap();
                uow.rollback().await.unwrap();

                '_test_block: {
                    let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                        ctx_manager.clone(),
                    )
                    .await;
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
            // TODO Subject to deletion
            let (ctx_manager, mut receiver) = ContextManager::new().await;

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

                // inject context
                let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                    ctx_manager.clone(),
                )
                .await;
                uow.begin().await.unwrap();
                uow.repository.add(&mut boardaggregate).await.unwrap();
                uow.commit().await.unwrap();

                '_test_block: {
                    let uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(
                        ctx_manager.clone(),
                    )
                    .await;
                    if let Err(err) = uow.repository.get(&id).await {
                        panic!("Fetch Error!:{}", err)
                    };
                    let mut count = 0;
                    'loo: loop {
                        match receiver.try_recv() {
                            Ok(_) => count += 1,
                            Err(_err) => break 'loo,
                        }
                    }

                    assert_eq!(count, 1);
                }
            }
        })
        .await;
    }
}
