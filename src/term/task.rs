use {
    std::{
        io,
        pin::Pin,
        process::ExitStatus,
        sync::atomic::Ordering,
    },
    futures::{
        task::{
            Context,
            Poll,
        },
        Future,
        FutureExt,
    },
    log::trace,
    crate::{
        ansi::Processor,
        screen::renderer::ScreenRendererNotifier,
    },
    super::Terminal,
};

struct TerminalTask<'a> {
    terminal: &'a Terminal,
    notifier: ScreenRendererNotifier,
    processor: Processor,
    must_notify: bool,
    buf: [u8; 512],
}

impl<'a> TerminalTask<'a> {
    pub fn new(terminal: &'a Terminal, notifier: ScreenRendererNotifier) -> Self {
        Self {
            terminal,
            notifier,
            processor: Processor::default(),
            must_notify: true,
            buf: [0; 512],
        }
    }
}

impl<'a> Future for TerminalTask<'a> {
    type Output = io::Result<ExitStatus>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let Self {
            ref terminal,
            ref mut notifier,
            ref mut processor,
            ref mut must_notify,
            ref mut buf,
        } = *self;
        if *must_notify {
            let mut notify = notifier.notify().boxed_local();
            match notify.poll_unpin(cx) {
                Poll::Ready(_) => {
                    *must_notify = false;
                },
                Poll::Pending => return Poll::Pending,
            };
        }
        let mut wait = terminal.wait();
        trace!("polling process");
        let res = match Pin::new(&mut wait).poll(cx) {
            Poll::Ready(exit_status) => {
                trace!("process finished");
                terminal.running.store(false, Ordering::SeqCst);
                Poll::Ready(exit_status)
            },
            Poll::Pending => {
                trace!("process still running");
                // If we don't wake this here, the runtime doesn't polls
                // this future again until the child exits.
                cx.waker().wake_by_ref();
                Poll::Pending
            },
        };
        if let Poll::Ready(stat) = res {
            return Poll::Ready(stat);
        }

        let mut read = terminal.pty_read(&mut buf[..]);
        trace!("polling pty reader");
        let res = match Pin::new(&mut read).poll(cx) {
            Poll::Ready(res) => {
                match res {
                    Ok(n) => {
                        trace!("read {} bytes from pty", n);
                        let mut screen = terminal.screen.lock().unwrap();
                        let mut pty = terminal.pty_writer.lock().unwrap();
                        processor.advance(
                            &buf[..n],
                            &mut *screen,
                            &mut *pty,
                        );
                        *must_notify = true;
                        // Wake so that we poll again and trigger the notifier.
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    },
                    Err(e) => {
                        trace!("error reading from pty: {}", e);
                        terminal.kill()?;
                        Poll::Ready(Err(e))
                    }
                }
            },
            Poll::Pending => {
                trace!("pty reader still pending");
                Poll::Pending
            },
        };

        res
    }
}

pub async fn terminal_task(terminal: &Terminal, notifier: ScreenRendererNotifier) -> io::Result<ExitStatus> {
    TerminalTask::new(terminal, notifier).await
}
