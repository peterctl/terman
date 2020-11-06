use {
    std::{
        io,
        sync::Arc,
    },
    tokio::sync::{
        mpsc::{
            channel,
            Sender,
            Receiver,
        },
    },
    log::trace,
    crate::{
        ansi::{
            self,
            Handler,
        },
        term::Terminal,
        util::Point,
        grid::Cell,
    },
};

pub fn renderer<W: io::Write>(
    terminal: Arc<Terminal>,
    writer: W,
    start: Point,
) -> (ScreenRenderer<W>, ScreenRendererNotifier)
{
    let (tx, rx) = channel(1);
    (
        ScreenRenderer { rx, terminal, writer, start },
        ScreenRendererNotifier { tx },
    )
}

pub struct ScreenRenderer<W: io::Write> {
    rx: Receiver<()>,
    terminal: Arc<Terminal>,
    writer: W,
    start: Point,
}

pub struct ScreenRendererNotifier {
    tx: Sender<()>,
}

impl ScreenRendererNotifier {
    pub async fn notify(&mut self) -> Result<(), ()> {
        self.tx
            .send(()).await
            .map_err(|_| ())
    }
}

impl<W: io::Write> ScreenRenderer<W> {
    pub async fn run_loop(&mut self) -> io::Result<()> {
        while let Some(()) = self.rx.recv().await {
            self.render_screen()?;
        }
        Ok(())
    }

    fn render_screen(&mut self) -> io::Result<()> {
        trace!("rendering screen");
        let screen = self.terminal.screen.lock().unwrap();
        let mut prev_attrs = None;
        let size = screen.size();
        writeln!(self.writer, "\x1b[2J\x1b[3J\x1b[{};{}H", self.start.y, self.start.x)?;
        write!(self.writer, "+")?;
        for _ in 0..screen.size().x {
            write!(self.writer, "-")?;
        }
        writeln!(self.writer, "+")?;
        for y in 0..size.y {
            write!(self.writer, "\x1b[{}G|", self.start.x)?;
            for x in 0..size.x {
                let ref cell = screen.cell(Point::new(x, y)).unwrap();
                if Some(&cell.attributes) != prev_attrs {
                    ansi::Renderer(&mut self.writer).render_attributes(&cell.attributes)?;
                }
                prev_attrs = Some(&cell.attributes);
                write!(self.writer, "{}", match cell.ch {
                    Some(c) => c,
                    None => ' ',
                })?;
            }
            writeln!(self.writer, "|")?;
        }
        write!(self.writer, "\x1b[{}G+", self.start.x)?;
        for _ in 0..screen.size().x {
            write!(self.writer, "-")?;
        }
        writeln!(self.writer, "+")?;
        let cursor = screen.cursor();
        let real_cursor = Point::new(
            cursor.x + self.start.x + 1,
            cursor.y + self.start.y + 2,
        );
        trace!("real cursor position: {:?}", real_cursor);
        write!(self.writer, "\x1b[{};{}H", real_cursor.y, real_cursor.x)?;
        self.writer.flush()?;
        Ok(())
    }
}
