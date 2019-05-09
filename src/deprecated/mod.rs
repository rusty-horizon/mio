#![allow(deprecated)]

mod event_loop;
mod io;
mod handler;
mod notify;

pub mod unix;

pub use self::event_loop::{
    EventLoop,
    EventLoopBuilder,
    Sender,
};
pub use self::io::{
    TryAccept,
    TryRead,
    TryWrite,
};
pub use self::handler::{
    Handler,
};
pub use self::notify::{
    NotifyError,
};
pub use self::unix::{
    pipe,
    PipeReader,
    PipeWriter,
    UnixListener,
    UnixSocket,
    UnixStream,
    Shutdown,
};
