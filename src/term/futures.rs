use {
    std::{
        io,
        pin::Pin,
        process::ExitStatus,
    },
    futures::{
        task::{
            Context,
            Poll,
        },
        Future,
    },
    tokio::io::{
        AsyncRead,
        AsyncWrite,
    },
    super::Terminal,
};

pub struct Read<'a> {
    pub terminal: &'a Terminal,
    pub buf: &'a mut [u8],
}

impl<'a> Future for Read<'a> {
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut pty = self.terminal.pty_reader.lock().unwrap();
        Pin::new(&mut *pty).poll_read(cx, self.buf)
    }
}

pub struct Write<'a> {
    pub terminal: &'a Terminal,
    pub buf: &'a [u8],
}

impl<'a> Future for Write<'a> {
    type Output = io::Result<usize>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut pty = self.terminal.pty_writer.lock().unwrap();
        Pin::new(&mut *pty).poll_write(cx, self.buf)
    }
}

pub struct Flush<'a> {
    pub terminal: &'a Terminal,
}

impl<'a> Future for Flush<'a> {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut pty = self.terminal.pty_writer.lock().unwrap();
        Pin::new(&mut *pty).poll_flush(cx)
    }
}

pub struct Shutdown<'a> {
    pub terminal: &'a Terminal,
}

impl<'a> Future for Shutdown<'a> {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut pty = self.terminal.pty_writer.lock().unwrap();
        Pin::new(&mut *pty).poll_shutdown(cx)
    }
}

pub struct Wait<'a> {
    pub terminal: &'a Terminal,
}

impl<'a> Future for Wait<'a> {
    type Output = io::Result<ExitStatus>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut process = self.terminal.process.lock().unwrap();
        Pin::new(&mut *process).poll(cx)
    }
}
