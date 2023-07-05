use std::{
    any::{Any, TypeId},
    collections::HashMap,
    env,
    sync::OnceLock,
};

use sqlx::{postgres::PgPoolOptions, PgPool};

use crate::{
    adapters::{database::AtomicContextManager, outbox::Outbox},
    domain::{
        board::{commands::*, events::BoardCreated},
        commands::ServiceResponse,
        Message,
    },
    services::handlers::{self, Future, ServiceHandler},
};
use crate::{services::messagebus::MessageBus, utils::ApplicationError};

pub struct Boostrap;
impl Boostrap {
    pub async fn message_bus() -> std::sync::Arc<MessageBus> {
        MessageBus::new(command_handler().await, event_handler().await)
    }
}

///* `Dependency` is the struct you implement injectable dependencies that will be called inside the function.

pub struct Dependency;

impl Dependency {
    pub fn some_dependency(&self) -> fn(String, i32) -> ServiceResponse {
        if cfg!(test) {
            |_: String, _: i32| -> ServiceResponse {
                println!("Some dependency invoked in test environment!");
                ServiceResponse::Empty(())
            }
        } else {
            |_: String, _: i32| -> ServiceResponse {
                println!("Not Test");
                ServiceResponse::Empty(())
            }
        }
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

static COMMAND_HANDLER: OnceLock<CommandHandler<AtomicContextManager>> = OnceLock::new();

pub async fn command_handler() -> &'static CommandHandler<AtomicContextManager> {
    let ch = match COMMAND_HANDLER.get() {
        None => {
            let command_handler = init_command_handler().await;

            COMMAND_HANDLER.get_or_init(|| command_handler)
        }
        Some(command_handler) => command_handler,
    };
    ch
}

static EVENT_HANDLER: OnceLock<EventHandler<AtomicContextManager>> = OnceLock::new();

pub async fn event_handler() -> &'static EventHandler<AtomicContextManager> {
    let eh = match EVENT_HANDLER.get() {
        None => {
            let event_handler = init_event_handler().await;

            EVENT_HANDLER.get_or_init(|| event_handler)
        }
        Some(event_handler) => event_handler,
    };
    eh
}

static POOL: OnceLock<PgPool> = OnceLock::new();

pub async fn connection_pool() -> &'static PgPool {
    let url = &env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let p = match POOL.get() {
        None => {
            let pool = PgPoolOptions::new()
                .max_connections(30)
                .connect(url)
                .await
                .map_err(|err| ApplicationError::DatabaseConnectionError(Box::new(err)))
                .unwrap();
            POOL.get_or_init(|| pool)
        }
        Some(pool) => pool,
    };
    p
}

static DEPENDENCY: OnceLock<Dependency> = OnceLock::new();

pub async fn dependency() -> &'static Dependency {
    let dp = match DEPENDENCY.get() {
        None => {
            let dependency = Dependency;

            DEPENDENCY.get_or_init(|| dependency)
        }
        Some(dependency) => dependency,
    };
    dp
}
