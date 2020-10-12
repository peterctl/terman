pub mod split;

use {
    std::{
        fs::File,
        io::{
            self,
            Read,
            Write,
        },
        os::unix::io::{
            AsRawFd,
            FromRawFd,
            RawFd,
        },
        pin::Pin,
        process::Stdio,
        task::{
            Context,
            Poll,
        },
    },
    anyhow::{
        Context as _,
        Result,
    },
    mio::{
        Evented,
        Poll as MioPoll,
        PollOpt,
        Ready,
        Token,
        unix::{
            EventedFd,
            UnixReady,
        },
    },
    tokio::{
        io::{
            AsyncRead,
            AsyncWrite,
            PollEvented,
        },
        process::Command,
    },
    termios::{
        TCSANOW,
        Termios,
        tcsetattr,
    },
    crate::util::Point,
};

pub use split::{
    split,
    PtyReader,
    PtyWriter,
};

#[inline]
fn wrap_io_err(error: bool) -> io::Result<()> {
    if error {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

pub struct PtyFile {
    file: File,
}

impl Evented for PtyFile {
    fn register(
        &self,
        poll: &MioPoll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()>
    {
        EventedFd(&self.file.as_raw_fd()).register(
            poll,
            token,
            interest | UnixReady::hup(),
            opts,
        )
    }

    fn reregister(
        &self,
        poll: &MioPoll,
        token: Token,
        interest: Ready,
        opts: PollOpt,
    ) -> io::Result<()>
    {
        EventedFd(&self.file.as_raw_fd()).reregister(
            poll,
            token,
            interest | UnixReady::hup(),
            opts,
        )
    }

    fn deregister(&self, poll: &MioPoll) -> io::Result<()> {
        EventedFd(&self.file.as_raw_fd()).deregister(poll)
    }
}

impl Read for PtyFile {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.file.read(buf)
    }
}

impl Write for PtyFile {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.file.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.file.flush()
    }
}

impl AsRawFd for PtyFile {
    fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

impl FromRawFd for PtyFile {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Self { file: File::from_raw_fd(fd) }
    }
}

pub struct Pty {
    inner: PollEvented<PtyFile>,
}

impl Pty {
    pub fn new<F: AsRawFd>(file: F) -> io::Result<Self> {
        if unsafe { libc::isatty(file.as_raw_fd()) == 0 } {
            Err(io::Error::from_raw_os_error(libc::ENOTTY))
        } else {
            Ok(Self {
                inner: PollEvented::new(
                    unsafe { PtyFile::from_raw_fd(file.as_raw_fd()) }
                )?
            })
        }
    }

    pub fn open() -> Result<Self> {
        let master = unsafe {
            const NONBLOCK_AFTER_OPEN: bool = cfg!(target_os = "freebsd");

            let flags = if NONBLOCK_AFTER_OPEN {
                libc::O_RDWR | libc::O_NOCTTY
            } else {
                libc::O_RDWR | libc::O_NOCTTY | libc::O_NONBLOCK
            };
            let flags = libc::O_RDWR | libc::O_NOCTTY;
            let master_fd = libc::posix_openpt(flags);
            wrap_io_err(master_fd < 0).context("open pty master")?;
            wrap_io_err(libc::grantpt(master_fd) != 0).context("grantpt")?;
            wrap_io_err(libc::unlockpt(master_fd) != 0).context("unlockpt")?;

            if NONBLOCK_AFTER_OPEN {
                let flags = libc::fcntl(master_fd, libc::F_GETFL, 0);
                wrap_io_err(flags < 0)?;
                wrap_io_err(
                    libc::fcntl(master_fd, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0
                )?;
            }

            PtyFile::from_raw_fd(master_fd)
        };

        Ok(Self {
            inner: PollEvented::new(master).context("create PollEvented")?
        })
    }

    pub fn open_slave(&self) -> Result<RawFd> {
        let slave_fd = unsafe {
            let mut buf = [0 as libc::c_char; 512];
            #[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
            {
                wrap_io_err(
                    libc::ptsname_r(self.as_raw_fd(), buf.as_mut_ptr(), buf.len()) != 0
                ).context("get pty name")?;
            }
            #[cfg(any(target_os = "macos", target_os = "freebsd"))]
            {
                let st = libc::ptsname(self.0);
                wrap_io_err(st.is_null()).context("get pty name")?;
                libc::strncpy(buf.as_mut_ptr(), st, buf.len());
            }

            let fd = libc::open(buf.as_ptr(), libc::O_RDWR);
            wrap_io_err(fd < 0).context("open slave fd")?;
            fd
        };
        Ok(slave_fd)
    }

    pub fn set_size(&mut self, size: Point) -> Result<()> {
        let winsize = libc::winsize {
            ws_col: size.x as u16,
            ws_row: size.y as u16,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };
        wrap_io_err(
            unsafe {
                libc::ioctl(self.as_raw_fd(), libc::TIOCSWINSZ, &winsize) == -1
            }
        ).context("set pty size")?;
        Ok(())
    }

    pub fn get_size(&self) -> Result<Point> {
        let mut winsize = libc::winsize {
            ws_col: 0,
            ws_row: 0,
            ws_xpixel: 0,
            ws_ypixel: 0,
        };

        wrap_io_err(
            unsafe {
                libc::ioctl(self.as_raw_fd(), libc::TIOCGWINSZ, &mut winsize) == -1
            }
        ).context("get pty size")?;
        Ok(Point::new(winsize.ws_col as usize, winsize.ws_row as usize))
    }

    pub fn set_mode(&mut self, mode: &Termios) -> Result<()> {
        tcsetattr(self.as_raw_fd(), TCSANOW, mode).context("tcsetattr")
    }

    pub fn get_mode(&self) -> Result<Termios> {
        Termios::from_fd(self.as_raw_fd()).context("tcgetattr")
    }
}

impl AsyncRead for Pty {
    fn poll_read(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        AsyncRead::poll_read(
            Pin::new(&mut self.inner),
            cx,
            buf,
        )
    }
}

impl AsyncWrite for Pty {
    fn poll_write(mut self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        AsyncWrite::poll_write(
            Pin::new(&mut self.inner),
            cx,
            buf,
        )
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        AsyncWrite::poll_flush(
            Pin::new(&mut self.inner),
            cx,
        )
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        AsyncWrite::poll_shutdown(
            Pin::new(&mut self.inner),
            cx,
        )
    }
}

impl AsRawFd for Pty {
    fn as_raw_fd(&self) -> RawFd {
        self.inner.get_ref().as_raw_fd()
    }
}

unsafe impl std::marker::Send for Pty {}

unsafe impl std::marker::Sync for Pty {}

pub trait WithPty {
    fn with_pty(&mut self, master: &Pty) -> Result<&mut Self>;
}

impl WithPty for Command {
    fn with_pty(&mut self, master: &Pty) -> Result<&mut Self> {
        let master_fd = master.as_raw_fd();
        let slave_fd = master.open_slave().context("pty open slave")?;
        unsafe {
            self
                .stdin(Stdio::from_raw_fd(slave_fd))
                .stdout(Stdio::from_raw_fd(slave_fd))
                .stderr(Stdio::from_raw_fd(slave_fd));

            self.pre_exec(move || {
                wrap_io_err(libc::close(master_fd) != 0)?;
                // wrap_io_err(libc::close(slave_fd) != 0)?;
                wrap_io_err(libc::setsid() < 0)?;
                wrap_io_err(libc::ioctl(slave_fd, libc::TIOCSCTTY.into(), 1) != 0)?;
                Ok(())
            });
        }

        Ok(self)
    }
}
