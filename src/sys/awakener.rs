pub use self::pipe::Awakener;

/// Default awakener backed by a pipe
mod pipe {
    use event::Evented;
    use std::io::{Read, Write};
    use sys;
    use {io, Interests, PollOpt, Registry, Token};

    /*
     *
     * ===== Awakener =====
     *
     */

    pub struct Awakener {
        reader: sys::Io,
        writer: sys::Io,
    }

    impl Awakener {
        pub fn new() -> io::Result<Awakener> {
            let (rd, wr) = sys::pipe()?;

            Ok(Awakener {
                reader: rd,
                writer: wr,
            })
        }

        pub fn wakeup(&self) -> io::Result<()> {
            match (&self.writer).write(&[1]) {
                Ok(_) => Ok(()),
                Err(e) => {
                    if e.kind() == io::ErrorKind::WouldBlock {
                        Ok(())
                    } else {
                        Err(e)
                    }
                }
            }
        }

        pub fn cleanup(&self) {
            let mut buf = [0; 128];

            loop {
                // Consume data until all bytes are purged
                match (&self.reader).read(&mut buf) {
                    Ok(i) if i > 0 => {}
                    _ => return,
                }
            }
        }

        fn reader(&self) -> &sys::Io {
            &self.reader
        }
    }

    impl Evented for Awakener {
        fn register(
            &self,
            registry: &Registry,
            token: Token,
            interests: Interests,
            opts: PollOpt,
        ) -> io::Result<()> {
            self.reader().register(registry, token, interests, opts)
        }

        fn reregister(
            &self,
            registry: &Registry,
            token: Token,
            interests: Interests,
            opts: PollOpt,
        ) -> io::Result<()> {
            self.reader().reregister(registry, token, interests, opts)
        }

        fn deregister(&self, registry: &Registry) -> io::Result<()> {
            self.reader().deregister(registry)
        }
    }
}
