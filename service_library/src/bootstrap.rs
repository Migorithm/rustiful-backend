use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{
        atomic::AtomicPtr,
        atomic::Ordering::{Acquire, Release},
    },
};

use crate::{
    adapters::{
        database::{AtomicConnection, Connection},
        outbox::Outbox,
    },
    domain::{board::commands::*, commands::ServiceResponse, Message},
    services::handlers::{self, Future, ServiceHandler},
};

pub struct Boostrap {
    pub connection: AtomicConnection,
    pub command_handler: &'static CommandHandler<AtomicConnection>,
    pub event_handler: &'static EventHandler<AtomicConnection>,
}
impl Boostrap {
    pub async fn new() -> Self {
        Self {
            connection: Connection::new().await.unwrap(),
            command_handler: command_handler(),
            event_handler: event_handler(),
        }
    }
}

pub type EventHandler<T> =
    HashMap<String, Vec<Box<dyn Fn(Box<dyn Message>, T) -> Future<ServiceResponse> + Send + Sync>>>;
pub type CommandHandler<T> = HashMap<
    TypeId,
    Box<dyn Fn(Box<dyn Any + Send + Sync>, T) -> Future<ServiceResponse> + Send + Sync>,
>;

macro_rules! command_handler {
    (
        [$iden:ident, $injectable:ty] ; {$($command:ty:$handler:expr),*}
    )
        => {
        pub fn init_command_handler() -> HashMap::<TypeId,Box<dyn Fn(Box<dyn Any + Send + Sync>, $injectable ) -> Future<ServiceResponse> + Send + Sync>>{
            let mut map: HashMap::<_,Box<dyn Fn(_, _ ) -> Future<_> + Send + Sync>> = HashMap::new();
            $(
                map.insert(
                    TypeId::of::<$command>(),
                    Box::new(
                        |c:Box<dyn Any+Send+Sync>, $iden: $injectable |->Future<ServiceResponse>{
                            $handler(*c.downcast::<$command>().unwrap(),$iden)
                        }
                    )
                );
            )*
            map
        }
    };
}

macro_rules! event_handler {
    (
        [$iden:ident, $injectable:ty] ; {$($event:ty: [$($handler:expr),* ]),*}
    ) =>{
        pub fn init_event_handler() -> HashMap<String, Vec<Box<dyn Fn(Box<dyn Message>, $injectable) -> Future<ServiceResponse> + Send + Sync>>>{
            let mut map : HashMap<String, Vec<Box<dyn Fn(_, _) -> Future<_> + Send + Sync>>> = HashMap::new();
            $(
                map.insert(
                    stringify!($event).into(),
                    vec![
                        $(
                            Box::new(
                                |e, $iden: $injectable| -> Future<ServiceResponse>{
                                    $handler(e,$iden)
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

command_handler!(
    [conn, AtomicConnection];
    {
        CreateBoard: ServiceHandler::create_board,
        EditBoard: ServiceHandler::edit_board,
        AddComment: ServiceHandler::add_comment,
        EditComment: ServiceHandler::edit_comment,
        Outbox: ServiceHandler::handle_outbox
    }
);
event_handler!(
    [conn,AtomicConnection];
    {
        BoardCreated : [handlers::EventHandler::test_event_handler,handlers::EventHandler::test_event_handler2]
    }
);

pub fn command_handler() -> &'static CommandHandler<AtomicConnection> {
    static PTR: AtomicPtr<CommandHandler<AtomicConnection>> = AtomicPtr::new(std::ptr::null_mut());
    let mut p = PTR.load(Acquire);

    if p.is_null() {
        p = Box::into_raw(Box::new(init_command_handler()));
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

pub fn event_handler() -> &'static EventHandler<AtomicConnection> {
    static PTR: AtomicPtr<EventHandler<AtomicConnection>> = AtomicPtr::new(std::ptr::null_mut());
    let mut p = PTR.load(Acquire);

    if p.is_null() {
        p = Box::into_raw(Box::new(init_event_handler()));
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
