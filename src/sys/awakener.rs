pub use self::pipe::Awakener;

/// Default awakener backed by a pipe
mod pipe {
    use crate::sys as unix;
    use crate::{io, Ready, Poll, PollOpt, Token};
    use crate::event::Evented;
    use std::io::{Read, Write};

    /*
     *
     * ===== Awakener =====
     *
     */

    pub struct Awakener {
        //reader: unix::Io,
        //writer: unix::Io,
    }

    impl Awakener {
        pub fn new() -> io::Result<Awakener> {
            Ok(Awakener {})
        }

        pub fn wakeup(&self) -> io::Result<()> {
            unimplemented!("not supported by horizon")
        }

        pub fn cleanup(&self) {
            unimplemented!("not supported by horizon")
        }

        fn reader(&self) -> &unix::Io {
            unimplemented!("not supported by horizon")
        }
    }

    impl Evented for Awakener {
        fn register(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
            //self.reader().register(poll, token, interest, opts)
            Ok(())
        }

        fn reregister(&self, poll: &Poll, token: Token, interest: Ready, opts: PollOpt) -> io::Result<()> {
            //self.reader().reregister(poll, token, interest, opts)
            Ok(())
        }

        fn deregister(&self, poll: &Poll) -> io::Result<()> {
            //self.reader().deregister(poll)
            Ok(())
        }
    }
}
