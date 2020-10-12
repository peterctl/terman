use std::fmt;
use std::str;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Color {
    Indexed(u8),
    Rgb(RgbColor),
    Special(SpecialColor),
}

#[allow(non_upper_case_globals)]
impl Color {
    pub const Black: Self = Self::Indexed(0);
    pub const Red: Self = Self::Indexed(1);
    pub const Green: Self = Self::Indexed(2);
    pub const Yellow: Self = Self::Indexed(3);
    pub const Blue: Self = Self::Indexed(4);
    pub const Magenta: Self = Self::Indexed(5);
    pub const Cyan: Self = Self::Indexed(6);
    pub const White: Self = Self::Indexed(7);

    pub const BrightBlack: Self = Self::Indexed(8);
    pub const BrightRed: Self = Self::Indexed(9);
    pub const BrightGreen: Self = Self::Indexed(10);
    pub const BrightYellow: Self = Self::Indexed(11);
    pub const BrightBlue: Self = Self::Indexed(12);
    pub const BrightMagenta: Self = Self::Indexed(13);
    pub const BrightCyan: Self = Self::Indexed(14);
    pub const BrightWhite: Self = Self::Indexed(15);

    pub const Foreground: Self = Self::Special(SpecialColor::Foreground);
    pub const Background: Self = Self::Special(SpecialColor::Background);
    pub const Cursor: Self = Self::Special(SpecialColor::Cursor);
}

impl str::FromStr for Color {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().replace("_", "-").as_str() {
            "black" => Ok(Self::Black),
            "red" => Ok(Self::Red),
            "green" => Ok(Self::Green),
            "yellow" => Ok(Self::Yellow),
            "blue" => Ok(Self::Blue),
            "magenta" => Ok(Self::Magenta),
            "cyan" => Ok(Self::Cyan),
            "white" => Ok(Self::White),
            "bright-black" => Ok(Self::BrightBlack),
            "bright-red" => Ok(Self::BrightRed),
            "bright-green" => Ok(Self::BrightGreen),
            "bright-yellow" => Ok(Self::BrightYellow),
            "bright-blue" => Ok(Self::BrightBlue),
            "bright-magenta" => Ok(Self::BrightMagenta),
            "bright-cyan" => Ok(Self::BrightCyan),
            "bright-white" => Ok(Self::BrightWhite),
            "foreground" => Ok(Self::Foreground),
            "background" => Ok(Self::Background),
            "cursor" => Ok(Self::Cursor),
            s => {
                if let Ok(rgb) = RgbColor::from_str(s) {
                    Ok(Self::Rgb(rgb))
                } else if let Ok(idx) = u8::from_str(s) {
                    Ok(Self::Indexed(idx))
                } else {
                    Err(())
                }
            }
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &Self::Black => write!(f, "black"),
            &Self::Red => write!(f, "red"),
            &Self::Green => write!(f, "green"),
            &Self::Yellow => write!(f, "yellow"),
            &Self::Blue => write!(f, "blue"),
            &Self::Magenta => write!(f, "magenta"),
            &Self::Cyan => write!(f, "cyan"),
            &Self::White => write!(f, "white"),
            &Self::BrightBlack => write!(f, "bright-black"),
            &Self::BrightRed => write!(f, "bright-red"),
            &Self::BrightGreen => write!(f, "bright-green"),
            &Self::BrightYellow => write!(f, "bright-yellow"),
            &Self::BrightBlue => write!(f, "bright-blue"),
            &Self::BrightMagenta => write!(f, "bright-magenta"),
            &Self::BrightCyan => write!(f, "bright-cyan"),
            &Self::BrightWhite => write!(f, "bright-white"),
            Self::Special(SpecialColor::Foreground) => write!(f, "foreground"),
            Self::Special(SpecialColor::Background) => write!(f, "background"),
            Self::Special(SpecialColor::Cursor) => write!(f, "cursor"),
            Self::Rgb(rgb) => fmt::Display::fmt(rgb, f),
            Self::Indexed(idx) => fmt::Display::fmt(idx, f),
        }
    }
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct RgbColor {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl str::FromStr for RgbColor {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.as_bytes() {
            [b'r', b'g', b'b', b':', s @ ..] => {
                // We use str::from_utf8_unchecked because we already know
                // this is valid UTF8.
                let chunks: Vec<_> = unsafe { str::from_utf8_unchecked(s) }
                    .split('/').map(|c| u8::from_str_radix(c, 16)).collect();
                if let &[Ok(r), Ok(g), Ok(b)] = &*chunks {
                    return Ok(Self { r, g, b });
                }
            }
            [b'#', s @ ..] | [b'0', b'x', s @ ..] => {
                // We use str::from_utf8_unchecked because we already know
                // this is valid UTF8.
                let s = unsafe { str::from_utf8_unchecked(s) };
                match s.len() {
                    2 => {
                        if let Ok(n) = u8::from_str_radix(s, 16) {
                            return Ok(RgbColor { r: n, g: n, b: n, });
                        }
                    },
                    6 => {
                        if let Ok(n) = u32::from_str_radix(s, 16) {
                            let r = ((n >> 16) & 0xff) as u8;
                            let g = ((n >> 8) & 0xff) as u8;
                            let b = (n & 0xff) as u8;
                            return Ok(RgbColor { r, g, b, });
                        }
                    },
                    _ => {},
                };
            },
            _ => {},
        };
        Err(())
    }
}

impl fmt::Display for RgbColor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:x}{:x}{:x}", self.r, self.g, self.b)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum SpecialColor {
    Foreground,
    Background,
    Cursor,
}

#[cfg(test)]
mod tests {
    use crate::ansi::color::{Color, RgbColor};

    #[test]
    fn test_color_parsing() {
        assert_eq!(
            Ok(Color::BrightYellow),
            "bright-yellow".parse(),
        );

        assert_eq!(
            Ok(Color::Rgb(RgbColor { r: 0xF0, g: 0xF0, b: 0xF0 })),
            "#F0".parse(),
        );

        assert_eq!(
            Ok(Color::Rgb(RgbColor { r: 0xF0, g: 0xF1, b: 0xF2 })),
            "#F0F1F2".parse(),
        );

        assert_eq!(
            Ok(Color::Rgb(RgbColor { r: 0x00, g: 0x88, b: 0xFF })),
            "rgb:00/88/ff".parse(),
        );

        assert_eq!(Ok(Color::Indexed(23)), "23".parse());
    }
}
