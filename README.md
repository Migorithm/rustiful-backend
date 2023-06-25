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
    type Event;
    fn collect_events(&mut self) -> VecDeque<Self::Event> {
        if !self.events().is_empty() {
            self.take_events()
        } else {
            VecDeque::new()
        }
    }
    fn events(&self) -> &VecDeque<Self::Event>;

    fn take_events(&mut self) -> VecDeque<Self::Event>;
    fn raise_event(&mut self, event: Self::Event);
}
```
And each event that's raised after handling command is internally registerd.
 

## Repository
`Repository` is to keep persistence concerns outside of the domain model.<br>
Its usability really shines in this project as I didn't use any ORM model.<br>
If your *business logic* is entangled with the *query logic*, it easily gets messy.<br>
You can use `Repository` pattern to steer clear of that situation.<br><br>

In this project, `one repository for one aggregate` priciple is strictly observed.<br>
So, every access to persistence layer is done through repository and its associated type, `Aggregate` as follows:
```rust
#[async_trait]
pub trait TRepository {
    type Aggregate: AsMut<Self::Aggregate>
        + AsRef<Self::Aggregate>
        + Aggregate<Event = Self::Event>
        + Send
        + Sync;
    type Event: Message;
...
}
```



### Repository - query logic
To insert, fetch and update aggregate, the following method must be implemented
```rust
pub trait TRepository {
    ...

    async fn add(
        &mut self,
        mut aggregate: impl AsMut<Self::Aggregate> + Send + Sync,
    ) -> Result<String, ApplicationError> {
        self._collect_events(aggregate.as_mut());
        self._add(aggregate.as_mut()).await
    }

    async fn _add(
        &mut self,
        aggregate: impl AsRef<Self::Aggregate> + Send + Sync,
    ) -> Result<String, ApplicationError>;

    async fn get(&mut self, aggregate_id: &str) -> Result<Self::Aggregate, ApplicationError>;

    async fn update(
        &mut self,
        mut aggregate: impl AsMut<Self::Aggregate> + Send + Sync,
    ) -> Result<(), ApplicationError> {
        self._collect_events(aggregate.as_mut());
        self._update(aggregate.as_mut()).await
    }

    async fn _update(
        &mut self,
        aggregate: impl AsRef<Self::Aggregate> + Send + Sync,
    ) -> Result<(), ApplicationError>;
}
```
As you can see, `_collect_events` event hook is followed by the query operations such as `_add` and `_update`.<br>
This is to collect events that are raised in domain logic so it can trigger the subsequent event handling logics.<br>

### Repository - event handling
```rust
pub trait TRepository {
    ...


    fn get_events(&self) -> &VecDeque<Self::Event>;
    fn set_events(&mut self, events: VecDeque<Self::Event>);

    fn _collect_events(&mut self, mut aggregate: impl AsMut<Self::Aggregate> + Send + Sync) {
        self.set_events(aggregate.as_mut().collect_events())
    }
    
}
```
Here, we have `get_events` and `set_events` methods that must be implemeted on any concrete repository that implements `TRepository` trait. If then, `_collect_event` is called on every `add` and `update` operation, as those operations are what requires data changes.<br><br>

Notice, however, that these events are for internal handling that happens to be side effects of command handling but not necessarily as important as command call. But what if the following event is as important so you can't just ignore the failure? Then we need to take more strigent data saving mechanism. The choice in this project is outbox pattern.

### Repository - outbox
```rust
pub trait TRepository {
    ...

  fn _collect_outbox(&self) -> Box<dyn Iterator<Item = Outbox> + '_ + Send> {
        Box::new(
            self.get_events()
                .iter()
                .filter(|e| e.externally_notifiable())
                .map(|e| e.outbox()),
        )
    }
}

```
Okay, it simply loops through the result of `get_events` method and filter in the event that's *externally_notifiable* and convert it to `Outbox` type. We will cover that later.










