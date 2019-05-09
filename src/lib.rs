#![doc(html_root_url = "https://docs.rs/mio/0.6.16")]
#![deny(missing_docs, missing_debug_implementations)]
#![cfg_attr(test, deny(warnings))]

//! A fast, low-level IO library for Rust focusing on non-blocking APIs, event
//! notification, and other useful utilities for building high performance IO
//! apps.
//!
//! # Goals
//!
//! * Fast - minimal overhead over the equivalent OS facilities (epoll, kqueue, etc...)
//! * Zero allocations
//! * A scalable readiness-based API, similar to epoll on Linux
//! * Design to allow for stack allocated buffers when possible (avoid double buffering).
//! * Provide utilities such as a timers, a notification channel, buffer abstractions, and a slab.
//!
//! # Platforms
//!
//! Currently supported platforms:
//!
//! * Linux
//! * OS X
//! * Windows
//! * FreeBSD
//! * NetBSD
//! * Android
//! * iOS
//!
//! mio can handle interfacing with each of the event notification systems of the aforementioned platforms. The details of
//! their implementation are further discussed in [`Poll`].
//!
//! # Usage
//!
//! Using mio starts by creating a [`Poll`], which reads events from the OS and
//! put them into [`Events`]. You can handle IO events from the OS with it.
//!
//! For more detail, see [`Poll`].
//!
//! [`Poll`]: struct.Poll.html
//! [`Events`]: struct.Events.html
//!
//! # Example
//!
//! ```
//! use mio::*;
//! use mio::net::{TcpListener, TcpStream};
//!
//! // Setup some tokens to allow us to identify which event is
//! // for which socket.
//! const SERVER: Token = Token(0);
//! const CLIENT: Token = Token(1);
//!
//! let addr = "127.0.0.1:13265".parse().unwrap();
//!
//! // Setup the server socket
//! let server = TcpListener::bind(&addr).unwrap();
//!
//! // Create a poll instance
//! let poll = Poll::new().unwrap();
//!
//! // Start listening for incoming connections
//! poll.register(&server, SERVER, Ready::readable(),
//!               PollOpt::edge()).unwrap();
//!
//! // Setup the client socket
//! let sock = TcpStream::connect(&addr).unwrap();
//!
//! // Register the socket
//! poll.register(&sock, CLIENT, Ready::readable(),
//!               PollOpt::edge()).unwrap();
//!
//! // Create storage for events
//! let mut events = Events::with_capacity(1024);
//!
//! loop {
//!     poll.poll(&mut events, None).unwrap();
//!
//!     for event in events.iter() {
//!         match event.token() {
//!             SERVER => {
//!                 // Accept and drop the socket immediately, this will close
//!                 // the socket and notify the client of the EOF.
//!                 let _ = server.accept();
//!             }
//!             CLIENT => {
//!                 // The server just shuts down the socket, let's just exit
//!                 // from our event loop.
//!                 return;
//!             }
//!             _ => unreachable!(),
//!         }
//!     }
//! }
//!
//! ```

extern crate lazycell;
extern crate net2;
extern crate iovec;
extern crate slab;

#[cfg(target_os = "fuchsia")]
extern crate fuchsia_zircon as zircon;
#[cfg(target_os = "fuchsia")]
extern crate fuchsia_zircon_sys as zircon_sys;

extern crate libc;

#[macro_use]
extern crate log;

mod event_imp;
mod io;
mod poll;
mod sys;
mod token;

pub mod net;

#[deprecated(since = "0.6.5", note = "use mio-extras instead")]
#[cfg(feature = "with-deprecated")]
#[doc(hidden)]
pub mod channel;

#[deprecated(since = "0.6.5", note = "use mio-extras instead")]
#[cfg(feature = "with-deprecated")]
#[doc(hidden)]
pub mod timer;

#[deprecated(since = "0.6.5", note = "update to use `Poll`")]
#[cfg(feature = "with-deprecated")]
#[doc(hidden)]
pub mod deprecated;

#[deprecated(since = "0.6.5", note = "use iovec crate directly")]
#[cfg(feature = "with-deprecated")]
#[doc(hidden)]
pub use iovec::IoVec;

#[deprecated(since = "0.6.6", note = "use net module instead")]
#[cfg(feature = "with-deprecated")]
#[doc(hidden)]
pub mod tcp {
    pub use net::{TcpListener, TcpStream};
    pub use std::net::Shutdown;
}

#[deprecated(since = "0.6.6", note = "use net module instead")]
#[cfg(feature = "with-deprecated")]
#[doc(hidden)]
pub mod udp;

pub use poll::{
    Poll,
    Registration,
    SetReadiness,
};
pub use event_imp::{
    PollOpt,
    Ready,
};
pub use token::Token;

pub mod event {
    //! Readiness event types and utilities.

    pub use super::poll::{Events, Iter};
    pub use super::event_imp::{Event, Evented};
}

pub use event::{
    Events,
};

#[deprecated(since = "0.6.5", note = "use events:: instead")]
#[cfg(feature = "with-deprecated")]
#[doc(hidden)]
pub use event::{Event, Evented};

#[deprecated(since = "0.6.5", note = "use events::Iter instead")]
#[cfg(feature = "with-deprecated")]
#[doc(hidden)]
pub use poll::Iter as EventsIter;

#[deprecated(since = "0.6.5", note = "std::io::Error can avoid the allocation now")]
#[cfg(feature = "with-deprecated")]
#[doc(hidden)]
pub use io::deprecated::would_block;

pub mod unix {
    //! Unix only extensions
    pub use sys::{
        EventedFd,
    };
    pub use sys::UnixReady;
}

#[cfg(target_os = "fuchsia")]
pub mod fuchsia {
    //! Fuchsia-only extensions
    //!
    //! # Stability
    //!
    //! This module depends on the [magenta-sys crate](https://crates.io/crates/magenta-sys)
    //! and so might introduce breaking changes, even on minor releases,
    //! so long as that crate remains unstable.
    pub use sys::{
        EventedHandle,
    };
    pub use sys::fuchsia::{FuchsiaReady, zx_signals_t};
}

#[cfg(feature = "with-deprecated")]
mod convert {
    use std::time::Duration;

    const NANOS_PER_MILLI: u32 = 1_000_000;
    const MILLIS_PER_SEC: u64 = 1_000;

    /// Convert a `Duration` to milliseconds, rounding up and saturating at
    /// `u64::MAX`.
    ///
    /// The saturating is fine because `u64::MAX` milliseconds are still many
    /// million years.
    pub fn millis(duration: Duration) -> u64 {
        // Round up.
        let millis = (duration.subsec_nanos() + NANOS_PER_MILLI - 1) / NANOS_PER_MILLI;
        duration.as_secs().saturating_mul(MILLIS_PER_SEC).saturating_add(millis as u64)
    }
}
