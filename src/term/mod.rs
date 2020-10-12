mod futures;
mod task;

use {
    std::{
        io,
        sync::{
            atomic::{
                AtomicBool,
                Ordering,
            },
            Mutex,
        },
    },
    anyhow::{
        Context as _,
        Result,
    },
    log::trace,
    tokio::process::{
        Child,
        Command,
    },
    crate::{
        pty::{
            Pty,
            PtyReader,
            PtyWriter,
            WithPty,
            split,
        },
        screen::Screen,
        util::Point,
    },
    self::futures as fut,
};

pub use task::terminal_task;

pub struct Terminal {
    // SAFETY: Pty operations are thread safe and so accessing them
    // from an immutable reference is safe.
    pub process: Mutex<Child>,
    pub pty_reader: Mutex<PtyReader>,
    pub pty_writer: Mutex<PtyWriter>,
    pub screen: Mutex<Screen>,
    pub running: AtomicBool,
    pub dirty: AtomicBool,
}

impl Terminal {
    pub fn spawn(mut command: Command, size: Point) -> Result<Self> {
        let mut pty = Pty::open().context("open pty")?;
        pty.set_size(size).context("pty set size")?;
        let process = command
            .with_pty(&pty).context("process add pty")?
            .spawn().context("spawn process")?;
        let (pty_reader, pty_writer) = split(pty);
        trace!("process spawned: {:?}", process);
        Ok(Self {
            process: Mutex::new(process),
            pty_reader: Mutex::new(pty_reader),
            pty_writer: Mutex::new(pty_writer),
            screen: Mutex::new(Screen::new(size)),
            running: AtomicBool::new(true),
            dirty: AtomicBool::new(true),
        })
    }

    pub fn alive(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn kill(&self) -> io::Result<()> {
        let mut process = self.process.lock().unwrap();
        self.running.store(false, Ordering::SeqCst);
        process.kill()
    }

    pub fn wait<'a>(&'a self) -> fut::Wait<'a> {
        fut::Wait {
            terminal: self,
        }
    }

    pub fn pty_read<'a>(&'a self, buf: &'a mut [u8]) -> fut::Read<'a> {
        fut::Read {
            terminal: self,
            buf,
        }
    }

    pub fn pty_write<'a>(&'a self, buf: &'a [u8]) -> fut::Write<'a> {
        fut::Write {
            terminal: self,
            buf,
        }
    }

    pub fn pty_flush<'a>(&'a self) -> fut::Flush<'a> {
        fut::Flush {
            terminal: self,
        }
    }

    pub fn pty_shutdown<'a>(&'a self) -> fut::Shutdown<'a> {
        fut::Shutdown {
            terminal: self,
        }
    }

    // pub async fn pty_read(&self, buf: &mut [u8]) -> io::Result<usize> {
        // let mut pty_reader = self.pty_reader.lock().await;
        // pty_reader.read(buf).await
    // }
//
    // pub async fn pty_write(&self, buf: &[u8]) -> io::Result<usize> {
        // let mut pty_writer = self.pty_writer.lock().await;
        // let n = pty_writer.write(buf).await?;
        // pty_writer.flush().await?;
        // Ok(n)
    // }
//
    // pub async fn pty_flush(&self) -> io::Result<()> {
        // let mut pty_writer = self.pty_writer.lock().await;
        // pty_writer.flush().await
    // }
//
    // pub async fn pty_shutdown(&self) -> io::Result<()> {
        // let mut pty_writer = self.pty_writer.lock().await;
        // pty_writer.shutdown().await
    // }
}
