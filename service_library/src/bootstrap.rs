use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{
        atomic::AtomicPtr,
        atomic::Ordering::{Acquire, Release},
    },
};

use sqlx::PgPool;

use crate::services::messagebus::MessageBus;
use crate::{
    adapters::{
        database::{connection_pool, AtomicContextManager},
        outbox::Outbox,
    },
    domain::{
        board::{commands::*, events::BoardCreated},
        commands::ServiceResponse,
        Message,
    },
    services::handlers::{self, Future, ServiceHandler},
};

pub struct Boostrap;
impl Boostrap {
    pub async fn message_bus() -> std::sync::Arc<MessageBus> {
        MessageBus::new(command_handler().await, event_handler().await)
    }
}

///* `Dependency` is the struct you implement injectable dependencies that will be called inside the function.

pub struct Dependency {
    pool: &'static PgPool,
}

impl Dependency {
    fn new(pool: &'static PgPool) -> Self {
        Self { pool }
    }
    pub fn pool(&self) -> &'static PgPool {
        self.pool
    }

    pub fn some_dependency(&self) -> fn(String, i32) -> ServiceResponse {
        |_: String, _: i32| -> ServiceResponse { ServiceResponse::Empty(()) }
    }
}

pub type EventHandler<T> =
    HashMap<String, Vec<Box<dyn Fn(Box<dyn Message>, T) -> Future<ServiceResponse> + Send + Sync>>>;
pub type CommandHandler<T> = HashMap<
    TypeId,
    Box<dyn Fn(Box<dyn Any + Send + Sync>, T) -> Future<ServiceResponse> + Send + Sync>,
>;

macro_rules! init_command_handler {
    (
        {$($command:ty:$handler:expr $(=>($($injectable:ident),*))? ),* }
    )
        => {
        pub async fn init_command_handler() -> HashMap::<TypeId,Box<dyn Fn(Box<dyn Any + Send + Sync>, AtomicContextManager) -> Future<ServiceResponse> + Send + Sync>>{
            let _dependency= dependency().await;

            let mut map: HashMap::<_,Box<dyn Fn(_, _ ) -> Future<_> + Send + Sync>> = HashMap::new();
            $(
                map.insert(
                    TypeId::of::<$command>(),
                    Box::new(
                        |c:Box<dyn Any+Send+Sync>, context_manager: AtomicContextManager|->Future<ServiceResponse>{
                            // * Convert event so event handler accepts not Box<dyn Message> but `event_happend` type of message.
                            // ! Logically, as it's from TypId of command, it doesn't make to cause an error.

                            $handler(
                                *c.downcast::<$command>().unwrap(),
                                context_manager,
                            $(
                                // * Injectable functions are added here.
                                $(dependency.$injectable(),)*
                            )?
                          )
                        },
                    )
                );
            )*
            map
        }
    };
}
macro_rules! init_event_handler {
    (
        {$($event:ty: [$($handler:expr $(=>($($injectable:ident),*))? ),* ]),*}
    ) =>{
        pub async fn init_event_handler() -> HashMap<String, Vec<Box<dyn Fn(Box<dyn Message>, AtomicContextManager) -> Future<ServiceResponse> + Send + Sync>>>{
            let dependency= dependency().await;
            let mut map : HashMap<String, Vec<Box<dyn Fn(_, _) -> Future<_> + Send + Sync>>> = HashMap::new();
            $(
                map.insert(
                    stringify!($event).into(),
                    vec![
                        $(
                            Box::new(
                                |e:Box<dyn Message>, context_manager:AtomicContextManager| -> Future<ServiceResponse>{
                                    $handler(
                                        // * Convert event so event handler accepts not Box<dyn Message> but `event_happend` type of message.
                                        // Safety:: client should access this vector of handlers by providing the corresponding event name
                                        // So, when it is followed, it logically doesn't make sense to cause an error.
                                        *e.downcast::<$event>().expect("Not Convertible!"), context_manager,
                                    $(
                                        // * Injectable functions are added here.
                                        $(dependency.$injectable(),)*
                                    )?
                                    )
                                }
                                ),
                        )*
                    ]
                );
            )*
            map
        }
    };
}

// * Among dependencies, `Connectable` dependencies shouldn't be injected sometimes because
// * its state is usually globally managed as in conneciton pool in RDBMS.
// * Therefore, it's adviable to specify connectables seperately.
init_command_handler!(
    {
        CreateBoard: ServiceHandler::create_board,
        EditBoard: ServiceHandler::edit_board,
        AddComment: ServiceHandler::add_comment,
        EditComment: ServiceHandler::edit_comment,
        Outbox: ServiceHandler::handle_outbox
    }
);

init_event_handler!(
    {
        BoardCreated : [
            handlers::EventHandler::test_event_handler=>(some_dependency),
            handlers::EventHandler::test_event_handler2
        ]
    }
);

pub async fn command_handler() -> &'static CommandHandler<AtomicContextManager> {
    static PTR: AtomicPtr<CommandHandler<AtomicContextManager>> =
        AtomicPtr::new(std::ptr::null_mut());
    let mut p = PTR.load(Acquire);

    if p.is_null() {
        p = Box::into_raw(Box::new(init_command_handler().await));
        if let Err(e) = PTR.compare_exchange(std::ptr::null_mut(), p, Release, Acquire) {
            // Safety: p comes from Box::into_raw right above
            // and wasn't whared with any other thread
            drop(unsafe { Box::from_raw(p) });
            p = e;
        }
    }
    // Safety: p is not null and points to a properly initialized value
    unsafe { &*p }
}

pub async fn event_handler() -> &'static EventHandler<AtomicContextManager> {
    static PTR: AtomicPtr<EventHandler<AtomicContextManager>> =
        AtomicPtr::new(std::ptr::null_mut());
    let mut p = PTR.load(Acquire);

    if p.is_null() {
        p = Box::into_raw(Box::new(init_event_handler().await));
        if let Err(e) = PTR.compare_exchange(std::ptr::null_mut(), p, Release, Acquire) {
            // Safety: p comes from Box::into_raw right above
            // and wasn't whared with any other thread
            drop(unsafe { Box::from_raw(p) });
            p = e;
        }
    }
    // Safety: p is not null and points to a properly initialized value
    unsafe { &*p }
}

pub async fn dependency() -> &'static Dependency {
    static PTR: AtomicPtr<Dependency> = AtomicPtr::new(std::ptr::null_mut());
    let mut p = PTR.load(Acquire);

    if p.is_null() {
        p = Box::into_raw(Box::new(Dependency::new(connection_pool().await)));
        if let Err(e) = PTR.compare_exchange(std::ptr::null_mut(), p, Release, Acquire) {
            // Safety: p comes from Box::into_raw right above
            // and wasn't whared with any other thread
            drop(unsafe { Box::from_raw(p) });
            p = e;
        }
    }
    // Safety: p is not null and points to a properly initialized value
    unsafe { &*p }
}
