use {
    std::io,
    log::warn,
    crate::ansi::{
        Attributes,
        Color,
        Flags,
    },
};

pub struct Renderer<'a, W: io::Write>(pub &'a mut W);

impl<'a, W: io::Write> Renderer<'a, W> {
    pub fn render_attributes(&mut self, attributes: &Attributes) -> io::Result<()> {
        // Start of CSI sequence
        write!(self.0, "\x1b[0")?;

        // Render flags
        if attributes.flags.contains(Flags::BOLD) {
            write!(self.0, ";1")?;
        }
        if attributes.flags.contains(Flags::DIM) {
            write!(self.0, ";2")?;
        }
        if attributes.flags.contains(Flags::ITALIC) {
            write!(self.0, ";3")?;
        }
        if attributes.flags.contains(Flags::UNDERLINE) {
            write!(self.0, ";4")?;
        }
        if attributes.flags.contains(Flags::BLINK_SLOW) {
            write!(self.0, ";5")?;
        }
        if attributes.flags.contains(Flags::BLINK_FAST) {
            write!(self.0, ";6")?;
        }
        if attributes.flags.contains(Flags::INVERSE) {
            write!(self.0, ";7")?;
        }
        if attributes.flags.contains(Flags::HIDDEN) {
            write!(self.0, ";8")?;
        }
        if attributes.flags.contains(Flags::STRIKEOUT) {
            write!(self.0, ";9")?;
        }

        // Render foreground color
        match attributes.fg {
            Color::Black => write!(self.0, ";30")?,
            Color::Red => write!(self.0, ";31")?,
            Color::Green => write!(self.0, ";32")?,
            Color::Yellow => write!(self.0, ";33")?,
            Color::Blue => write!(self.0, ";34")?,
            Color::Magenta => write!(self.0, ";35")?,
            Color::Cyan => write!(self.0, ";36")?,
            Color::White => write!(self.0, ";37")?,
            Color::Rgb(c) => write!(self.0, ";38;2;{};{};{}", c.r, c.g, c.b)?,
            Color::Indexed(c) => write!(self.0, ";38;5;{}", c)?,
            Color::Foreground => write!(self.0, ";39")?,
            c => {
                warn!("invalid foreground color: {:?}", c);
                write!(self.0, ";30")?;
            },
        }

        // Render background color
        match attributes.bg {
            Color::Black => write!(self.0, ";40")?,
            Color::Red => write!(self.0, ";41")?,
            Color::Green => write!(self.0, ";42")?,
            Color::Yellow => write!(self.0, ";43")?,
            Color::Blue => write!(self.0, ";44")?,
            Color::Magenta => write!(self.0, ";45")?,
            Color::Cyan => write!(self.0, ";46")?,
            Color::White => write!(self.0, ";47")?,
            Color::Rgb(c) => write!(self.0, ";48;2;{};{};{}", c.r, c.g, c.b)?,
            Color::Indexed(c) => write!(self.0, ";48;5;{}", c)?,
            Color::Background => write!(self.0, ";49")?,
            c => {
                warn!("invalid background color: {:?}", c);
                write!(self.0, ";40")?;
            },
        }

        write!(self.0, "m")?;

        Ok(())
    }
}

