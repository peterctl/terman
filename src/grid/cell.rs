use crate::ansi::color::Color;

#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub ch: Option<char>,
    pub attributes: Attributes,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            ch: None,
            attributes: Attributes::default(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Attributes {
    pub fg: Color,
    pub bg: Color,
    pub flags: Flags,
}

impl Default for Attributes {
    fn default() -> Self {
        Self {
            fg: Color::Foreground,
            bg: Color::Background,
            flags: Flags::empty(),
        }
    }
}

bitflags::bitflags! {
    pub struct Flags: u16 {
        const INVERSE           = 0b0000_0000_0001;
        const BOLD              = 0b0000_0000_0010;
        const ITALIC            = 0b0000_0000_0100;
        const UNDERLINE         = 0b0000_0000_1000;
        const WRAPLINE          = 0b0000_0001_0000;
        const WIDE_CHAR         = 0b0000_0010_0000;
        const WIDE_CHAR_SPACER  = 0b0000_0100_0000;
        const DIM               = 0b0000_1000_0000;
        const HIDDEN            = 0b0001_0000_0000;
        const STRIKEOUT         = 0b0010_0000_0000;
        const BOLD_ITALIC       = Self::BOLD.bits | Self::ITALIC.bits;
        const BOLD_DIM          = Self::BOLD.bits | Self::DIM.bits;
    }
}
