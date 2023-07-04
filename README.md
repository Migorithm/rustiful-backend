## Motivation
For the last 2 years, I've found myself so much into MSA and distributed transactions.<br>
Dealing with edge cases and failover, I realized, unlike what people often say, performance is the key<br>
To reduce technical failure because at the end of the day, you don't get to make a mess on your application code<br>
But rather, failure happens when network is not stable and instances go down and so on, meaning that<br>
the faster you handle traffics, the better and more reliable the system gets.<br><br>

On top of this performance, I got interested in Rust's ownership system as it is very must different in handling<br>
pointers and lifetime of values. Added to that, I got to learn that Rust type system is by far the saftest<br>
with its low level performance.<br><br>

So the last question remaining was, how fast I could make system which assumes both performance and safety.<br>
And here is the experiment.<br>

## Architecture
In this project, I deliberately chose `Event Driven Architecture` with adoption of `Domain Driven Design`.<br>
Each transaction that may require subsequent transaction both internally and externally will be handled<br>
by leaving `outbox` message together in one transaction. So, there are three key architectural concerns covered<br>
- Event Driven Architecture
- Domain Driven Design
- Outbox Pattern


## Persistence
Due to the immaturity of asynchronous ORM, I decided not to use Diesel(or its async variant) but SQLx.<br>
As claimed, the notion of `Aggregate` taked from DDD community, transaction boundary is made strictly by<br>
consistency boundary that is set by aggregate.
```rust
pub trait Aggregate: Send + Sync {
    fn collect_events(&mut self) -> VecDeque<Box<dyn Message>> {
        if !self.events().is_empty() {
            self.take_events()
        } else {
            VecDeque::new()
        }
    }
    fn events(&self) -> &VecDeque<Box<dyn Message>>;

    fn take_events(&mut self) -> VecDeque<Box<dyn Message>>;
    fn raise_event(&mut self, event: Box<dyn Message>);
}

```
And each event that's raised after handling command is internally registerd.
 

## Repository
`Repository` is to keep persistence concerns outside of the domain model.<br>
Its usability really shines in this project as I didn't use any ORM model.<br>
If your *business logic* is entangled with the *query logic*, it easily gets messy.<br>
You can use `Repository` pattern to steer clear of that situation.<br><br>

In this project, `one repository for one aggregate` priciple is strictly observed.<br>
So, every access to persistence layer is done through repository and its generic type, `Aggregate` as follows:
```rust
#[async_trait]
pub trait TRepository<A: Aggregate + 'static>{
...
}
```



### Repository - query logic
To insert, fetch and update aggregate, the following method must be implemented
```rust
pub trait TRepository {
    ...

    async fn add(&mut self, aggregate: &mut A) -> Result<String, ApplicationError> {
        self._collect_events(aggregate);
        self._add(aggregate).await
    }

    async fn _add(&mut self, aggregate: &A) -> Result<String, ApplicationError>;

    async fn get(&self, aggregate_id: &str) -> Result<A, ApplicationError>;

    async fn update(&mut self, aggregate: &mut A) -> Result<(), ApplicationError> {
        self._collect_events(aggregate);
        self._update(aggregate).await
    }

    async fn _update(&mut self, aggregate: &A) -> Result<(), ApplicationError>;

}
```
As you can see, `_collect_events` event hook is followed by the query operations such as `_add` and `_update`.<br>
This is to collect events that are raised in domain logic so it can trigger the subsequent event handling logics.<br>

### Repository - event handling
```rust
pub trait TRepository {
    ...

    fn get_events(&mut self) -> VecDeque<Box<dyn Message>>;
    fn set_events(&mut self, events: VecDeque<Box<dyn Message>>);

    fn _collect_events(&mut self, aggregate: &mut A) {
        self.set_events(aggregate.collect_events())
    }
    
}
```
Here, we have `get_events` and `set_events` methods that must be implemeted on any concrete repository that implements `TRepository` trait. If then, `_collect_event` is called on every `add` and `update` operation, as those operations are what requires data changes.<br><br>

Notice, however, that these events are for internal handling that happens to be side effects of command handling but not necessarily as important as command call. But what if the following event is as important so you can't just ignore the failure? Then we need to take more strigent data saving mechanism. The choice in this project is outbox pattern.<br>

For that, we need higher level of abstraction for transaction management - `UnitOfWork`.<br><br>

## UnitOfWork
Unit of work is what guarantees an atomic transaction WITHIN a service handler. Service handler is what provides a certain service to the end user which is also referred to as application handler. So, it must hold a connection to storage of your interest and access to repository: 
```rust
// library::services::unit_of_work.rs
pub struct UnitOfWork<R, A>
where
    R: TRepository<A>,
    A: Aggregate + 'static,
{
    executor: Arc<RwLock<Executor>>,
    context: AtomicContextManager,
    repository: R,
    _aggregate: PhantomData<A>,
}
```

Here, `UnitOfWork` has generic parameter `R` and `A` which represents `Repository` and `Aggregate` respectively and holds `context` and `executor`. Leaving aside `context` and `executor`, every time a client needs to access repository, it needs to go through `UnitOfWork` as follows:
```rust
...
    pub fn repository(&mut self) -> &mut R {
        &mut self.repository
    }
```
<br><br>

More importantly, you may have multiple access to repository for various reasons. Regardlessly, every operation should be within one and only one atomic transaction. For that, `UnitOfWork` assumes a number of methods that emulates database-related operations:
```rust
    pub async fn begin(&mut self) -> Result<(), ApplicationError> {
        // TODO Need to be simplified
        let mut executor = self.executor.write().await;
        executor.begin().await
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
```

Also note that everytime commit or rollback is invoked, that's the end of `UnitOfWork` too. You can tell it by the signature thereof.<br><br>

### Commit Hook on UnitOfWork - Sending or Saving Events
Again, events must be persisted. And the probably the most appropriate place for that would be right before the `commit` operation. As you may have noticed, we have `_commit_hook` just for that.

```rust
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
```
Here, `context` shows up again and it actually has sender out of channel and its other paring receiver is listening to the channel. We will talk about that later. The rest of the codes is about sorting out the event type and deciding whether or not it is saved in the form of `outbox` or sent to channel. 
<br><br>

## From high level view - Service Handler
We've come a long way. Let's look at how this whole service is used from bird's eye view:
```rust
pub struct ServiceHandler;
impl ServiceHandler {
    pub fn create_board(
        cmd: CreateBoard,
        context: AtomicContextManager,
    ) -> Future<ServiceResponse> {
        Box::pin(async move {
            let mut uow = UnitOfWork::<Repository<BoardAggregate>, BoardAggregate>::new(context.clone()).await;
            uow.begin().await.unwrap();

            let builder = BoardAggregate::builder();
            let mut board_aggregate: BoardAggregate = builder.build();
            board_aggregate.create_board(cmd);
            let res = uow.repository().add(&mut board_aggregate).await?;

            uow.commit().await?;
            Ok(res.into())
        })
    }

    ...
}
```
Inside async block, we firstly initialize `UnitOfWork` given the context with `context` which plays a role of managing whole lifespan of local session and sending message to receiver. Transaction `begin` follows, and `commit` is placed right before returning the result. Between the `begin` and `commit` operation, we have lines of code that processes business logic.<br><br>


## Event Driven Architecture
It wouldn't be really useful to have such a rather complicated system if the whole thing was for simple request-and-response system. At the heart of event driven architecture, there is 'what if' mindset. For example, you want to implement checkout service which calls `payment service`. Within a payment service however, you often have a number of steps we need to go through such as:
- PG transaction
- Point system call
- Coupon service call
- Notifying delivery service upon the finalization of the process
<br><br>

None of them is not business critical, which means that you want to gear yourself up for failover. Even without failover, the simple call for `payment service` is not actually one-off call. That means, you want to have a central controller that manages: 
- handling `command`, the first call from enduser
- collecting of events
- assigning the events to the appropriate handlers
- collecting subsequent events about both happy and fail cases 

To achieve that, we can think of things like `Saga` or `Process Manager` but those are one of these things you can think of only when the service grows beyond the certain level. For this project, I assume everything is handled within the application. Nonethless, the architecture adopted in this project just suffices enough to explain how it should work even for the larger system. And here, we have `MessageBus`


## MessageBus
```rust
// library::services::messagebus.rs
pub struct MessageBus {
    #[cfg(test)]
    pub book_keeper: AtomicI32,

    command_handler: &'static CommandHandler<AtomicContextManager>,
    event_handler: &'static EventHandler<AtomicContextManager>,
}
```
Okay, `MessageBus` holds static reference to both command handler(s) and event handler(s). Firstly let's see `handle` method, the entry to any request to the any service in your application. 

### MessageBus - handle 
```rust
impl MessageBus {
...

    pub async fn handle<C>(&self, message: C) -> ApplicationResult<ServiceResponse>
        where
        C: Command + AnyTrait,
    {
        let (context_manager, mut event_receiver) = ContextManager::new().await;

        let res = self
            .command_handler
            .get(&message.type_id())
            .ok_or_else(|| {
                eprintln!("Unprocessable Command Given!");
                ApplicationError::CommandNotFound
            })?(message.as_any(), context_manager.clone())
        .await?;

        'event_handling_loop: loop {
            // * use of try_recv is to stop blocking it when all events are drained.
            match event_receiver.try_recv() {
                // * Logging!
                Ok(msg) => {
                    if let Err(ApplicationError::EventNotFound) =
                        self.handle_event(msg, context_manager.clone()).await
                    {
                        continue;
                    };
                }
                Err(TryRecvError::Empty) => {
                    if Arc::strong_count(&context_manager) == 1 {
                        break 'event_handling_loop;
                    } else {
                        continue;
                    }
                }
                Err(TryRecvError::Disconnected) => break 'event_handling_loop,
            };
        }
        drop(context_manager);
        Ok(res)
    }
}
```
If, `ContextManager` shows up again, and as explained, the initialization of this object is PER TRAFFIC, so it wouldn't cause *'logical'* race condition and the like. And the initialization returns not only `ContextManager` but also event_receiver(or listener). With the command argument you pass to this method, you process command first and in the service, as we investigated before, events are collected, sorted out and some of them are sent to the receiver. The next step is therefore taking out the events and calling `handle_event` method. If nothing is sent, it ends the loop and return the result you get from `command_handler`. 
<br><br>

``


### MessageBus - handle_event
```rust
    async fn handle_event(
        &self,
        msg: Box<dyn Message>,
        context_manager: AtomicContextManager,
    ) -> ApplicationResult<()> {
        // ! msg.topic() returns the name of event. It is crucial that it corresponds to the key registered on Event Handler.

        let handlers = self
            .event_handler
            .get(&msg.metadata().topic)
            .ok_or_else(|| {
                eprintln!("Unprocessable Event Given! {:?}", msg);
                ApplicationError::EventNotFound
            })?;

        for handler in handlers.iter() {
            match handler(msg.message_clone(), context_manager.clone()).await {
                Err(ApplicationError::StopSentinel) => {
                    eprintln!("Stop Sentinel Reached!");
                    break;
                }
                Err(err) => {
                    eprintln!("Error Occurred While Handling Event! Error:{}", err);
                }
                Ok(_val) => {
                    println!("Event Handling Succeeded!");
                }
            };

            #[cfg(test)]
            {
                self.book_keeper
                    .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }
        drop(context_manager);
        Ok(())
    }

```
Unlike command, event may have a number of attatched handlers so iterating over the handlers given the message topic is required. For the specific handler, you may go through failure after which you should stop the subsequent event handling. For that, Your application may want to have `StopSentinel` error and when that is caught, you just break out of the loop. Logically, before sending `StopSentinel`, if failover handling is necessary, the sentinel raising handler should issue another event for those cases, which will be handled with another set of handlers attached to overcome the failure. 




