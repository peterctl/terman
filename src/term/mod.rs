mod processor;
mod terminal;

pub use terminal::Terminal;
pub use processor::Processor;

pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

pub struct Size {
    pub lines: usize,
    pub columns: usize,
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

pub enum ClearScreenMode {
    Below,
    Above,
    All,
    Saved,
}

pub enum ClearLineMode {
    Right,
    Left,
    All,
}
