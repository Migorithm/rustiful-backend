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
 




