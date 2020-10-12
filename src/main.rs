#![feature(core_intrinsics)]
#![feature(slice_index_methods)]

extern crate futures;
extern crate libc;
extern crate log;
extern crate simple_logging;
extern crate termios;
extern crate tokio;
extern crate vte;

mod ansi;
mod grid;
#[macro_use] mod macros;
mod pty;
mod screen;
mod term;
mod util;

use {
    std::{
        io,
        sync::Arc,
        os::unix::io::{
            RawFd,
            AsRawFd,
        },
        time::Duration,
    },
    anyhow::{
        Context,
        Result,
    },
    log::trace,
    tokio::{
        io::{
            self as aio,
            AsyncReadExt,
        },
        process::Command,
        runtime::Builder as RuntimeBuilder,
        task,
    },
    termios::*,
    crate::{
        pty::Pty,
        screen::renderer::renderer,
        term::{
            Terminal,
            terminal_task,
        },
    },
};

fn set_raw_terminal(fd: i32) -> io::Result<Termios> {
    let old_tios = Termios::from_fd(fd)?;
    let mut new_tios = old_tios.clone();
    // cfmakeraw(&mut new_tios);
    new_tios.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    new_tios.c_iflag &= !(BRKINT | ICRNL | INPCK | ISTRIP | IXON);
    new_tios.c_cflag &= !(CSIZE | PARENB);
    new_tios.c_cflag |= CS8;
    new_tios.c_oflag = !(OPOST);

    // new_tios.c_iflag = ICRNL|IXANY;
    // new_tios.c_oflag = OPOST|ONLCR;
    // new_tios.c_cflag = CREAD|CS8|HUPCL;
    new_tios.c_cc[VMIN] = 1;
    new_tios.c_cc[VTIME] = 0;
    cfsetispeed(&mut new_tios, cfgetispeed(&old_tios))?;
    cfsetospeed(&mut new_tios, cfgetospeed(&old_tios))?;
    tcsetattr(fd, TCSANOW, &new_tios)?;
    Ok(old_tios)
}

fn set_non_blocking(fd: RawFd) -> io::Result<()> {
    unsafe {
        let flags = libc::fcntl(fd, libc::F_GETFL, 0);
        if flags < 0 {
            return Err(io::Error::last_os_error());
        }
        let ret = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);
        if ret == -1 {
            return Err(io::Error::last_os_error());
        }

        Ok(())
    }
}

async fn async_main() -> Result<()> {
    simple_logging::log_to_file("terman.log", log::LevelFilter::Trace)?;

    let mut stdin = Pty::new(std::io::stdin()).context("stdin is not a pty")?;
    let old_tios = set_raw_terminal(stdin.as_raw_fd())?;
    let current_size = stdin.get_size().context("get stdin size")?;
    let slave_size = current_size / 2;
    let start_point = slave_size / 2;
    let command = Command::new("/bin/bash");
    let terminal = Terminal::spawn(command, slave_size).context("create terminal")?;
    let terminal = Arc::new(terminal);
    trace!("starting loop");
    let (mut renderer, notifier) = renderer(
        Arc::clone(&terminal),
        io::stdout(),
        start_point,
    );
    let render_screen = {
        let terminal = Arc::clone(&terminal);
        task::spawn(async move {
            trace!("starting render task");
            let res = renderer.run_loop().await;
            trace!("render task finished");
            if let Err(_) = res {
                terminal.kill()?;
            }
            res
        })
    };
    let stdin_read = {
        let terminal = Arc::clone(&terminal);
        task::spawn(async move {
            let mut buf = [0u8; 128];
            while terminal.alive() {
                trace!("reading from stdin");
                let n = stdin.read(&mut buf[..]).await.context("stdin read")?;
                trace!("read {} bytes from stdin", n);
                trace!("writing terminal pty");
                let res = terminal.pty_write(&buf[..n]).await;
                terminal.pty_flush().await?;
                trace!("wrote terminal pty");
                match res {
                    Ok(_) => {},
                    Err(e) if e.kind() == io::ErrorKind::Interrupted => return Ok(()),
                    Err(e) => return Err(e).context("pty write"),
                }
            };
            trace!("stdin read task finished");
            Ok(())
        })
    };

    let exit_status = terminal_task(&*terminal, notifier).await;
    drop(stdin_read);
    drop(render_screen);
    tcsetattr(aio::stdin().as_raw_fd(), TCSANOW, &old_tios)?;
    print!("\x1b[2J\x1b[3J\x1b[H"); // Clear screen
    println!("process exit: {}", exit_status?);
    println!("there are {} references to terminal", Arc::strong_count(&terminal));
    Ok(())
}

fn main() -> Result<()> {
    let mut runtime = RuntimeBuilder::new()
        .threaded_scheduler()
        .max_threads(4)
        .core_threads(2)
        .enable_all()
        .build().context("create tokio runtime")?;

    runtime.block_on(async_main())?;
    runtime.shutdown_timeout(Duration::from_secs(0));
    trace!("runtime was shutdown");

    Ok(())
}
