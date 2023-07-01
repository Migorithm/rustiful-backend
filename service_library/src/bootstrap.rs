use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::{
        atomic::AtomicPtr,
        atomic::Ordering::{Acquire, Release},
    },
};

use crate::{
    adapters::{database::AtomicConnection, outbox::Outbox},
    domain::{board::commands::*, commands::ServiceResponse, Message},
    services::{
        handlers::{self, Future, ServiceHandler},
        messagebus::MessageBus,
    },
};

pub struct Boostrap;
impl Boostrap {
    pub async fn message_bus() -> std::sync::Arc<MessageBus> {
        MessageBus::new(command_handler(), event_handler()).await
    }
}

///* `Dependency` is the struct you implement injectable dependencies that will be called inside the function.

pub struct Dependency;

impl Dependency {
    // ! Test Dependency Used in `test_event_handler` - subject to being augmented
    fn some_dependency(_arg1: String, _arg2: i32) -> ServiceResponse {
        ServiceResponse::Empty(())
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
        [$connectable_ident:ident, $connectable:ty] ; {$($command:ty:$handler:expr $(=>($($injectable:ident),*))? ),* }
    )
        => {
        pub fn init_command_handler() -> HashMap::<TypeId,Box<dyn Fn(Box<dyn Any + Send + Sync>, $connectable ) -> Future<ServiceResponse> + Send + Sync>>{


            let mut map: HashMap::<_,Box<dyn Fn(_, _ ) -> Future<_> + Send + Sync>> = HashMap::new();
            $(
                map.insert(
                    TypeId::of::<$command>(),
                    Box::new(
                        |c:Box<dyn Any+Send+Sync>, $connectable_ident: $connectable |->Future<ServiceResponse>{
                            $handler(*c.downcast::<$command>().unwrap(),$connectable_ident,
                            $(
                                $(Box::new(Dependency::$injectable),)*
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
macro_rules! event_handler {
    (
        [$connectable_ident:ident, $connectable:ty] ; {$($event:ty: [$($handler:expr $(=>($($injectable:ident),*))? ),* ]),*}
    ) =>{
        pub fn init_event_handler() -> HashMap<String, Vec<Box<dyn Fn(Box<dyn Message>, $connectable) -> Future<ServiceResponse> + Send + Sync>>>{
            let mut map : HashMap<String, Vec<Box<dyn Fn(_, _) -> Future<_> + Send + Sync>>> = HashMap::new();
            $(
                map.insert(
                    stringify!($event).into(),
                    vec![
                        $(
                            Box::new(
                                |e, $connectable_ident: $connectable| -> Future<ServiceResponse>{
                                    $handler(e,$connectable_ident,
                                    $(
                                        $(Box::new(Dependency::$injectable),)*
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
        BoardCreated : [
            handlers::EventHandler::test_event_handler=>(some_dependency),
            handlers::EventHandler::test_event_handler2
            ]
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
