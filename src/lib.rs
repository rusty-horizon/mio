#![doc(html_root_url = "https://docs.rs/mio/0.6.16")]
#![deny(missing_docs, missing_debug_implementations)]
#![cfg_attr(test, deny(warnings))]
// Many of mio's public methods violate this lint, but they can't be fixed
// without a breaking change.
#![cfg_attr(feature = "cargo-clippy", allow(clippy::trivially_copy_pass_by_ref))]

//! A fast, low-level IO library for Rust focusing on non-blocking APIs, event
//! notification, and other useful utilities for building high performance IO
//! apps.
//!
//! # Features
//!
//! * Non-blocking TCP, UDP
//! * I/O event notification queue backed by epoll, kqueue, and IOCP
//! * Zero allocations at runtime
//! * Platform specific extensions
//!
//! # Non-goals
//!
//! The following are specifically omitted from Mio and are left to the user or higher-level libraries.
//!
//! * File operations
//! * Thread pools / multi-threaded event loop
//! * Timers
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
//! let mut poll = Poll::new().unwrap();
//! let registry = poll.registry().clone();
//!
//! // Start listening for incoming connections
//! registry.register(
//!     &server,
//!     SERVER,
//!     Interests::readable(),
//!     PollOpt::edge()).unwrap();
//!
//! // Setup the client socket
//! let sock = TcpStream::connect(&addr).unwrap();
//!
//! // Register the socket
//! registry.register(
//!     &sock,
//!     CLIENT,
//!     Interests::readable(),
//!     PollOpt::edge()).unwrap();
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

extern crate iovec;
extern crate net2;
extern crate slab;

#[cfg(unix)]
extern crate libc;

#[macro_use]
extern crate log;

mod event_imp;
mod io;
mod lazycell;
mod poll;
mod sys;
mod token;

pub mod net;

pub use event_imp::{Interests, PollOpt, Ready};
pub use poll::{Poll, Registration, Registry, SetReadiness};
pub use token::Token;

pub mod event {
    //! Readiness event types and utilities.

    pub use super::event_imp::{Event, Evented};
    pub use super::poll::{Events, Iter};
}

pub use event::Events;

#[cfg(unix)]
pub mod unix {
    //! Unix only extensions
    pub use sys::UnixReady;
    pub use sys::EventedFd;
}
