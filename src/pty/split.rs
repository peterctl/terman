use {
    std::{
        io::{
            self,
            Read,
            Write,
        },
        cell::UnsafeCell,
        pin::Pin,
        sync::Arc,
    },
    futures::{
        task::{
            Context,
            Poll,
        },
    },
    tokio::{
        io::{
            AsyncRead,
            AsyncWrite,
        },
    },
    super::Pty,
};

// EventedFd is thread safe as long as there's only one reader and one writer
// doing operations on it in parallel. Since `split` consumes the pty and returns
// only one reader and one writer, and the reader and writer in turn require a
// mutable reference in order to read and write, this function enforces EventedFd's
// safety requirements through the Rust type system.
pub fn split(pty: Pty) -> (PtyReader, PtyWriter) {
    let inner = Arc::new(UnsafeCell::new(pty));
    (
        PtyReader { inner: inner.clone() },
        PtyWriter { inner },
    )
}


pub struct PtyReader {
    inner: Arc<UnsafeCell<Pty>>,
}

impl Read for PtyReader {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let pty = unsafe { &mut *self.inner.as_ref().get() };
        pty.inner.get_mut().read(buf)
    }
}

impl AsyncRead for PtyReader {
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &mut [u8]) -> Poll<io::Result<usize>> {
        // SAFETY: Pty is thread safe as long as it has only a single reader and writer.
        // This is enforced by the type system in `split`.
        let pty = unsafe { &mut *self.inner.as_ref().get() };
        Pin::new(pty).poll_read(cx, buf)
    }
}

// SAFETY: Pty is thread safe as long as it has only a single reader and writer.
// This is enforced by the type system in `split`.
unsafe impl std::marker::Send for PtyReader {}

pub struct PtyWriter {
    inner: Arc<UnsafeCell<Pty>>,
}

impl Write for PtyWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let pty = unsafe { &mut *self.inner.as_ref().get() };
        pty.inner.get_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let pty = unsafe { &mut *self.inner.as_ref().get() };
        pty.inner.get_mut().flush()
    }
}

impl AsyncWrite for PtyWriter {
    fn poll_write(self: Pin<&mut Self>, cx: &mut Context<'_>, buf: &[u8]) -> Poll<io::Result<usize>> {
        // SAFETY: Pty is thread safe as long as it has only a single reader and writer.
        // This is enforced by the type system in `split`.
        let pty = unsafe { &mut *self.inner.as_ref().get() };
        Pin::new(pty).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // SAFETY: Pty is thread safe as long as it has only a single reader and writer.
        // This is enforced by the type system in `split`.
        let pty = unsafe { &mut *self.inner.as_ref().get() };
        Pin::new(pty).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        // SAFETY: Pty is thread safe as long as it has only a single reader and writer.
        // This is enforced by the type system in `split`.
        let pty = unsafe { &mut *self.inner.as_ref().get() };
        Pin::new(pty).poll_shutdown(cx)
    }
}

// SAFETY: Pty is thread safe as long as it has only a single reader and writer.
// This is enforced by the type system in `split`.
unsafe impl std::marker::Send for PtyWriter {}
