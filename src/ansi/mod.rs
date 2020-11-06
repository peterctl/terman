pub mod attributes;
pub mod charset;
pub mod color;
pub mod handler;
pub mod processor;
pub mod renderer;
pub mod sgr;

pub use {
    attributes::{
        Attributes,
        Flags,
    },
    charset::{
        CharsetList,
        CharsetIndex,
        StandardCharset,
    },
    color::{
        Color,
        RgbColor,
        SpecialColor,
    },
    handler::Handler,
    processor::Processor,
    renderer::Renderer,
};


/// C0 set of 7-bit control characters (from ANSI X3.4-1977).
#[allow(non_snake_case, dead_code)]
pub mod C0 {
    /// Null filler, terminal should ignore this character.
    pub const NUL: u8 = 0x00;
    /// Start of Header.
    pub const SOH: u8 = 0x01;
    /// Start of Text, implied end of header.
    pub const STX: u8 = 0x02;
    /// End of Text, causes some terminal to respond with ACK or NAK.
    pub const ETX: u8 = 0x03;
    /// End of Transmission.
    pub const EOT: u8 = 0x04;
    /// Enquiry, causes terminal to send ANSWER-BACK ID.
    pub const ENQ: u8 = 0x05;
    /// Acknowledge, usually sent by terminal in response to ETX.
    pub const ACK: u8 = 0x06;
    /// Bell, triggers the bell, buzzer, or beeper on the terminal.
    pub const BEL: u8 = 0x07;
    /// Backspace, can be used to define overstruck characters.
    pub const BS: u8 = 0x08;
    /// Horizontal Tabulation, move to next predetermined position.
    pub const HT: u8 = 0x09;
    /// Linefeed, move to same position on next line (see also NL).
    pub const LF: u8 = 0x0A;
    /// Vertical Tabulation, move to next predetermined line.
    pub const VT: u8 = 0x0B;
    /// Form Feed, move to next form or page.
    pub const FF: u8 = 0x0C;
    /// Carriage Return, move to first character of current line.
    pub const CR: u8 = 0x0D;
    /// Shift Out, switch to G1 (other half of character set).
    pub const SO: u8 = 0x0E;
    /// Shift In, switch to G0 (normal half of character set).
    pub const SI: u8 = 0x0F;
    /// Data Link Escape, interpret next control character specially.
    pub const DLE: u8 = 0x10;
    /// (DC1) Terminal is allowed to resume transmitting.
    pub const XON: u8 = 0x11;
    /// Device Control 2, causes ASR-33 to activate paper-tape reader.
    pub const DC2: u8 = 0x12;
    /// (DC2) Terminal must pause and refrain from transmitting.
    pub const XOFF: u8 = 0x13;
    /// Device Control 4, causes ASR-33 to deactivate paper-tape reader.
    pub const DC4: u8 = 0x14;
    /// Negative Acknowledge, used sometimes with ETX and ACK.
    pub const NAK: u8 = 0x15;
    /// Synchronous Idle, used to maintain timing in Sync communication.
    pub const SYN: u8 = 0x16;
    /// End of Transmission block.
    pub const ETB: u8 = 0x17;
    /// Cancel (makes VT100 abort current escape sequence if any).
    pub const CAN: u8 = 0x18;
    /// End of Medium.
    pub const EM: u8 = 0x19;
    /// Substitute (VT100 uses this to display parity errors).
    pub const SUB: u8 = 0x1A;
    /// Prefix to an escape sequence.
    pub const ESC: u8 = 0x1B;
    /// File Separator.
    pub const FS: u8 = 0x1C;
    /// Group Separator.
    pub const GS: u8 = 0x1D;
    /// Record Separator (sent by VT132 in block-transfer mode).
    pub const RS: u8 = 0x1E;
    /// Unit Separator.
    pub const US: u8 = 0x1F;
    /// Delete, should be ignored by terminal.
    pub const DEL: u8 = 0x7f;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CursorStyle {
    Default,
    BlinkingBlock,
    StaticBlock,
    BlinkingUnderline,
    StaticUnderline,
    BlinkingBar,
    StaticBar,
}

impl Default for CursorStyle {
    fn default() -> Self {
        Self::Default
    }
}

impl CursorStyle {
    pub fn from_primitive(number: i64) -> Option<Self> {
        match number {
            0 => Some(Self::Default),
            1 => Some(Self::BlinkingBlock),
            2 => Some(Self::StaticBlock),
            3 => Some(Self::BlinkingUnderline),
            4 => Some(Self::StaticUnderline),
            5 => Some(Self::BlinkingBar),
            6 => Some(Self::StaticBar),
            _ => None,
        }
    }

    pub fn to_primitive(&self) -> i64 {
        match self {
            Self::Default => 0,
            Self::BlinkingBlock => 1,
            Self::StaticBlock => 2,
            Self::BlinkingUnderline => 3,
            Self::StaticUnderline => 4,
            Self::BlinkingBar => 5,
            Self::StaticBar => 6,
        }
    }

    pub fn to_blinking(&self) -> Self {
        match self {
            Self::StaticBlock => Self::BlinkingBlock,
            Self::StaticUnderline => Self::BlinkingUnderline,
            Self::StaticBar => Self::BlinkingBar,
            style => *style,
        }
    }

    pub fn to_static(&self) -> Self {
        match self {
            Self::BlinkingBlock => Self::StaticBlock,
            Self::BlinkingUnderline => Self::StaticUnderline,
            Self::BlinkingBar => Self::StaticBar,
            style => *style,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClipboardType {
    Clipboard,
    Primary,
    Selection,
    Cut0,
    Cut1,
    Cut2,
    Cut3,
    Cut4,
    Cut5,
    Cut6,
    Cut7,
}

impl Default for ClipboardType {
    fn default() -> Self {
        Self::Clipboard
    }
}

impl ClipboardType {
    pub fn from_primitive(byte: u8) -> Option<Self> {
        match byte {
            b'c' => Some(ClipboardType::Clipboard),
            b'p' => Some(ClipboardType::Primary),
            b's' => Some(ClipboardType::Selection),
            b'0' => Some(ClipboardType::Cut0),
            b'1' => Some(ClipboardType::Cut1),
            b'2' => Some(ClipboardType::Cut2),
            b'3' => Some(ClipboardType::Cut3),
            b'4' => Some(ClipboardType::Cut4),
            b'5' => Some(ClipboardType::Cut5),
            b'6' => Some(ClipboardType::Cut6),
            b'7' => Some(ClipboardType::Cut7),
            _ => None,
        }
    }

    pub fn to_primitive(&self) -> u8 {
        match self {
            ClipboardType::Clipboard => b'c',
            ClipboardType::Primary => b'p',
            ClipboardType::Selection => b's',
            ClipboardType::Cut0 => b'0',
            ClipboardType::Cut1 => b'1',
            ClipboardType::Cut2 => b'2',
            ClipboardType::Cut3 => b'3',
            ClipboardType::Cut4 => b'4',
            ClipboardType::Cut5 => b'5',
            ClipboardType::Cut6 => b'6',
            ClipboardType::Cut7 => b'7',
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TerminalMode {
    CursorKeys,
    ColumnMode,
    Insert,
    Origin,
    LineWrap,
    BlinkingCursor,
    LineFeedNewLine,
    ShowCursor,
    ReportMouseClicks,
    ReportCellMouseMotion,
    ReportAllMouseMotion,
    ReportFocusInOut,
    Utf8Mouse,
    SgrMouse,
    AlternateScroll,
    SwapScreenAndSetRestoreCursor,
    BracketedPaste,
}

impl TerminalMode {
    pub fn from_primitive(intermediate: Option<&u8>, number: i64) -> Option<Self> {
        let private = match intermediate {
            Some(b'?') => true,
            None => false,
            _ => return None,
        };

        match (number, private) {
            (1, true) => Some(Self::CursorKeys),
            (3, true) => Some(Self::ColumnMode),
            (6, true) => Some(Self::Origin),
            (7, true) => Some(Self::LineWrap),
            (12, true) => Some(Self::BlinkingCursor),
            (25, true) => Some(Self::ShowCursor),
            (1000, true) => Some(Self::ReportMouseClicks),
            (1002, true) => Some(Self::ReportCellMouseMotion),
            (1003, true) => Some(Self::ReportAllMouseMotion),
            (1004, true) => Some(Self::ReportFocusInOut),
            (1005, true) => Some(Self::Utf8Mouse),
            (1006, true) => Some(Self::SgrMouse),
            (1007, true) => Some(Self::AlternateScroll),
            (1049, true) => Some(Self::SwapScreenAndSetRestoreCursor),
            (2004, true) => Some(Self::BracketedPaste),

            (4, false) => Some(Self::Insert),
            (20, false) => Some(Self::LineFeedNewLine),

            _ => None,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ClearScreenMode {
    Below,
    Above,
    All,
    Saved,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ClearLineMode {
    Right,
    Left,
    All,
}
