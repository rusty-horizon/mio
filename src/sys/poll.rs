use std::os::unix::io::RawFd;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;
use std::{cmp, i32};
use std::io;
use std::sync::Mutex;

use libc::{POLLERR, POLLHUP, POLLIN, POLLOUT, POLLPRI};

use crate::event_imp::Event;
use super::{cvt, UnixReady};
use crate::{PollOpt, Ready, Token};

/// Each Selector has a globally unique(ish) ID associated with it. This ID
/// gets tracked by `TcpStream`, `TcpListener`, etc... when they are first
/// registered with the `Selector`. If a type that is previously associated with
/// a `Selector` attempts to register itself with a different `Selector`, the
/// operation will return with an error. This matches windows behavior.
static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

pub struct Selector {
    id: usize,
    events: Mutex<Vec<libc::pollfd>>,
}

impl Selector {
    pub fn new() -> io::Result<Selector> {
        // offset by 1 to avoid choosing 0 as the id of a selector
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed) + 1;

        Ok(Selector { id: id, events: Mutex::new(vec![]) })
    }

    pub fn id(&self) -> usize {
        self.id
    }

    /// Wait for events from the OS
    pub fn select(
        &self,
        evts: &mut Events,
        awakener: Token,
        timeout: Option<Duration>,
    ) -> io::Result<bool> {
        let timeout_ms = timeout
            .map(|to| cmp::min(millis(to), i32::MAX as u64) as i32)
            .unwrap_or(-1);

        evts.events.clear();

        let mut events = self.events.lock().unwrap();

        if unsafe { cvt(libc::poll(
            events.as_mut_ptr(),
            events.len() as u32,
            timeout_ms,
        ))? } != 0 {
            for (i, event) in events.iter_mut().enumerate() {
                if event.revents == 0 { continue; }

                if i == awakener.into() {
                    event.revents = 0;
                    return Ok(true);
                } else {
                    evts.events.push(*event);
                    event.revents = 0;
                }
            }
        }

        Ok(false)
    }

    /// Register event interests for the given IO handle with the OS
    pub fn register(
        &self,
        fd: RawFd,
        token: Token,
        interests: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        let info = libc::pollfd {
            fd: usize::from(token) as i32,
            events: ready_to_poll(interests, opts),
            revents: 0,
        };

        self.events.lock().unwrap().push(info);

        Ok(())
    }

    /// Register event interests for the given IO handle with the OS
    pub fn reregister(
        &self,
        fd: RawFd,
        token: Token,
        interests: Ready,
        opts: PollOpt,
    ) -> io::Result<()> {
        let info = libc::pollfd {
            fd: fd as i32,
            events: ready_to_poll(interests, opts),
            revents: 0,
        };

        self.events.lock().unwrap()[usize::from(token)] = info;

        Ok(())
    }

    /// Deregister event interests for the given IO handle with the OS
    pub fn deregister(&self, fd: RawFd) -> io::Result<()> {
        self.events.lock().unwrap().retain(|e| e.fd != fd);
        Ok(())
    }
}

fn ready_to_poll(interest: Ready, opts: PollOpt) -> i16 {
    let mut kind = 0;

    if interest.is_readable() {
        kind |= POLLIN;
    }

    if interest.is_writable() {
        kind |= POLLOUT;
    }

    if UnixReady::from(interest).is_priority() {
        kind |= POLLPRI;
    }

    kind as i16
}

pub struct Events {
    events: Vec<libc::pollfd>,
}

impl Events {
    pub fn with_capacity(u: usize) -> Events {
        Events {
            events: Vec::with_capacity(u),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.events.capacity()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    #[inline]
    pub fn get(&self, idx: usize) -> Option<Event> {
        self.events.get(idx).map(|event| {
            let epoll = event.revents;
            let mut kind = Ready::empty();

            if (epoll & POLLIN) != 0 {
                kind = kind | Ready::readable();
            }

            if (epoll & POLLPRI) != 0 {
                kind = kind | Ready::readable() | UnixReady::priority();
            }

            if (epoll & POLLOUT) != 0 {
                kind = kind | Ready::writable();
            }

            // EPOLLHUP - Usually means a socket error happened
            if (epoll & POLLERR) != 0 {
                kind = kind | UnixReady::error();
            }

            if (epoll & POLLHUP) != 0 {
                kind = kind | UnixReady::hup();
            }

            Event::new(kind, Token(idx))
        })
    }

    pub fn push_event(&mut self, event: Event) {
        if self.events.len() < event.token().into() {
            self.events.resize(event.token().into(), libc::pollfd {
                fd: 0,
                events: 0,
                revents: 0,
            });
        }

        self.events[usize::from(event.token())] = libc::pollfd {
            fd: 0,
            events: ready_to_poll(event.readiness(), PollOpt::empty()),
            revents: ready_to_poll(event.readiness(), PollOpt::empty()),
        };
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }
}

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
    duration
        .as_secs()
        .saturating_mul(MILLIS_PER_SEC)
        .saturating_add(millis as u64)
}
