mod poll;

pub use self::poll::{Events, Selector};

mod awakener;
mod eventedfd;
mod io;
mod ready;
mod tcp;
mod udp;

#[cfg(feature = "with-deprecated")]
mod uds;

pub use self::awakener::Awakener;
pub use self::eventedfd::EventedFd;
pub use self::io::{Io, set_nonblock};
pub use self::ready::{UnixReady, READY_ALL};
pub use self::tcp::{TcpStream, TcpListener};
pub use self::udp::UdpSocket;

#[cfg(feature = "with-deprecated")]
pub use self::uds::UnixSocket;

pub use iovec::IoVec;

use std::os::unix::io::FromRawFd;

pub fn pipe() -> crate::io::Result<(Io, Io)> {
    let mut pipes = [0; 2];
    let flags = libc::O_NONBLOCK | libc::O_CLOEXEC;
    unsafe {
        cvt(libc::pipe(pipes.as_mut_ptr()))?;
        libc::fcntl(pipes[0], libc::F_SETFL, flags);
        libc::fcntl(pipes[1], libc::F_SETFL, flags);
    }

    unsafe {
        Ok((Io::from_raw_fd(pipes[0]), Io::from_raw_fd(pipes[1])))
    }
}

trait IsMinusOne {
    fn is_minus_one(&self) -> bool;
}

impl IsMinusOne for i32 {
    fn is_minus_one(&self) -> bool { *self == -1 }
}
impl IsMinusOne for isize {
    fn is_minus_one(&self) -> bool { *self == -1 }
}

fn cvt<T: IsMinusOne>(t: T) -> crate::io::Result<T> {
    use std::io;

    if t.is_minus_one() {
        Err(io::Error::last_os_error())
    } else {
        Ok(t)
    }
}
