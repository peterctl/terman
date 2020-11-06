use std::str;

use crate::ansi::{
    color::{
        SpecialColor,
        RgbColor,
    },
    sgr,
    C0,
    charset::{
        CharsetIndex,
        StandardCharset,
    },
};
use super::{
    Terminal,
    CursorStyle,
    ClipboardType,
    ClearLineMode,
    ClearScreenMode,
    TerminalMode,
};

pub struct Processor {
    state: ProcessorState,
    parser: vte::Parser,
}

impl Default for Processor {
    fn default() -> Self {
        Self {
            state: ProcessorState::default(),
            parser: vte::Parser::new(),
        }
    }
}

impl Processor {
    pub fn advance(
        &mut self,
        bytes: &[u8],
        terminal: &mut dyn Terminal,
        writer: &mut dyn std::io::Write,
    ) {
        let mut performer = Performer::new(terminal, writer, &mut self.state);
        for byte in bytes {
            self.parser.advance(&mut performer, *byte);
        }
    }
}

pub struct ProcessorState {
    preceding_char: Option<char>,
}

impl Default for ProcessorState {
    fn default() -> Self {
        Self { preceding_char: None }
    }
}

pub struct Performer<'a> {
    terminal: &'a mut dyn Terminal,
    writer: &'a mut dyn std::io::Write,
    state: &'a mut ProcessorState,
}

impl<'a> Performer<'a> {
    pub fn new(
        terminal: &'a mut dyn Terminal,
        writer: &'a mut dyn std::io::Write,
        state: &'a mut ProcessorState,
    ) -> Self {
        Self {
            terminal,
            writer,
            state,
        }
    }
}

impl<'a> ::vte::Perform for Performer<'a> {
    fn print(&mut self, ch: char) {
        self.terminal.put_char(ch);
        self.state.preceding_char = Some(ch);
    }

    fn execute(&mut self, byte: u8) {
        match byte {
            C0::BEL => self.terminal.bell(),
            C0::BS => self.terminal.put_backspace(1),
            C0::HT => self.terminal.put_tab(1),
            C0::LF | C0::VT | C0::FF => self.terminal.put_lf(),
            C0::CR => self.terminal.put_cr(),
            C0::SI => self.terminal.set_active_charset(CharsetIndex::G0),
            C0::SO => self.terminal.set_active_charset(CharsetIndex::G1),
            _ => {
                // TODO unhandled
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        match (byte, intermediates.get(0)) {
            (b'0', Some(b'(')) => self.terminal.configure_charset(CharsetIndex::G0, StandardCharset::Ascii),
            (b'0', Some(b')')) => self.terminal.configure_charset(CharsetIndex::G1, StandardCharset::Ascii),
            (b'7', None) => self.terminal.save_cursor_position(),
            (b'8', None) => self.terminal.restore_cursor_position(),
            (b'8', Some(b'#')) => self.terminal.do_alignment_test(),
            (b'=', None) => self.terminal.set_keypad_application_mode(),
            (b'>', None) => self.terminal.unset_keypad_application_mode(),
            (b'B', Some(b'(')) => self.terminal.configure_charset(CharsetIndex::G0, StandardCharset::Special),
            (b'B', Some(b')')) => self.terminal.configure_charset(CharsetIndex::G1, StandardCharset::Special),
            (b'D', None) => self.terminal.put_lf(),
            (b'E', None) => {
                self.terminal.put_lf();
                self.terminal.put_cr();
            },
            (b'H', None) => self.terminal.set_horizontal_tabstop(
                self.terminal.cursor().x,
            ),
            (b'M', None) => self.terminal.reverse_index(),
            (b'c', None) => self.terminal.reset_state(),
            (b'\\', None) => {},
            _ => {
                // TODO implement unhandled.
            }
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        macro_rules! unhandled {
            () => {{
                // TODO
            }};
        }

        if params.is_empty() || params[0].is_empty() {
            unhandled!();
            return;
        }

        let terminal = &mut self.terminal;
        let writer = &mut self.writer;
        let terminator = if bell_terminated { '\x07' } else { '\x1b' };
        let num = match str::from_utf8(params[0]).ok()
                .and_then(|s| s.parse::<u32>().ok()) {
            Some(n) => n,
            None => {
                unhandled!();
                return;
            }
        };

        match num {
            // Set window title
            0 | 2 => {
                if params.len() > 1 {
                    // flat_map for Result<T> yields T if Ok(T) or None if Err.
                    // It's basically a shorthand for map and filter.
                    let title = params[1..].iter()
                        .flat_map(|b| str::from_utf8(b))
                        .collect::<Vec<&str>>()
                        .join(";");
                    terminal.set_title(title.trim());
                } else {
                    unhandled!();
                }
            },

            // Set color
            4 => {
                if params.len() != 3 {
                    unhandled!();
                    return;
                }

                let index: Option<u8> = str::from_utf8(params[1]).ok()
                    .and_then(|s| s.parse().ok());
                let color: Option<RgbColor> = str::from_utf8(params[2]).ok()
                    .and_then(|s| s.parse().ok());

                if let (Some(index), Some(color)) = (index, color) {
                    terminal.set_color(index, color);
                }
            },

            // Set path
            7 => {
                if params.len() <= 1 {
                    unhandled!();
                    return;
                }

                // flat_map for Result<T> yields T if Ok(T) or None if Err.
                // It's basically a shorthand for map and filter.
                let path = params[1..].iter()
                    .flat_map(|b| str::from_utf8(b))
                    .collect::<Vec<&str>>()
                    .join(";");

                terminal.set_path(path.trim());
            },

            // Set/get foreground/background/cursor color
            10 | 11 | 12 => {
                if params.len() < 2 {
                    unhandled!();
                    return;
                }

                let mut i = num;

                for param in &params[1..] {
                    let index = match i {
                        10 => SpecialColor::Foreground,
                        11 => SpecialColor::Background,
                        12 => SpecialColor::Cursor,
                        _ => {
                            break;
                        },
                    };
                    i += 1;

                    match *param {
                        b"?" => {
                            if let Some(ref color) = terminal.get_special_color(index) {
                                write!(
                                    writer,
                                    "\x1b]{};rgb:{1:02x}{1:02x}/{2:02x}{2:02x}/{3:02x}{3:02x}{4}",
                                    i,
                                    color.r,
                                    color.g,
                                    color.b,
                                    terminator,
                                );
                            }
                        },
                        s => {
                            let color: Option<RgbColor> = str::from_utf8(s)
                                .ok().and_then(|s| s.parse().ok());
                            if let Some(color) = color {
                                terminal.set_special_color(index, color);
                            } else {
                                unhandled!();
                                return;
                            }
                        }
                    }
                }
            },

            // Set cursor style
            50 => {
                if params.len() >= 2
                    && params[1].len() >= 13
                    && params[1][0..12] == *b"CursorShape="
                {
                    let style: Option<i64> = str::from_utf8(&params[1][12..])
                        .ok().and_then(|s| s.parse().ok());
                    let style = match style {
                        Some(n) => n,
                        None => {
                            unhandled!();
                            return;
                        }
                    };
                    if let Some(style) = CursorStyle::from_primitive(style) {
                        terminal.set_cursor_style(style);
                    } else {
                        unhandled!();
                    }
                } else {
                    unhandled!();
                }
            },

            // Clipboard operations
            52 => {
                if params.len() != 3 {
                    unhandled!();
                    return;
                }

                let clipboard_char = params[1].get(0).map(|c| *c).unwrap_or(b'c');
                let clipboard_type = match ClipboardType::from_primitive(clipboard_char) {
                    Some(clipboard_type) => clipboard_type,
                    _ => {
                        unhandled!();
                        return;
                    },
                };

                match params[2] {
                    b"?" => {
                        if let Some(ref data) = terminal.get_clipboard(clipboard_type) {
                            let data = base64::encode(data);
                            write!(writer, "\x1b]52;{};{}{}", clipboard_char, data, terminator);
                        }
                    },
                    data => {
                        if let Ok(data) = base64::decode(data) {
                            terminal.set_clipboard(clipboard_type, data.as_ref());
                        } else {
                            unhandled!();
                        }
                    },
                };
            },

            // Reset color
            104 => {
                if params.len() == 1 {
                    for index in 0..=255 {
                        terminal.reset_color(index);
                    }
                }
                let index: Option<u8> = str::from_utf8(params[1]).ok()
                    .and_then(|s| s.parse().ok());
                if let Some(index) = index {
                    terminal.reset_color(index);
                }
            },

            // Reset foreground color
            110 => {
                terminal.reset_special_color(SpecialColor::Foreground);
            },

            // Reset background color
            111 => {
                terminal.reset_special_color(SpecialColor::Background);
            },

            // Reset cursor color
            112 => {
                terminal.reset_special_color(SpecialColor::Cursor);
            },

            _ => unhandled!(),
        }
    }

    fn csi_dispatch(
        &mut self,
        args: &[i64],
        intermediates: &[u8],
        has_ignored_intermediates: bool,
        action: char,
    ) {
        macro_rules! unhandled {
            () => {{
                // TODO
            }};
        }

        macro_rules! get_arg {
            (idx: $index:expr, def: $default:expr) => {
                args.get($index)
                    .and_then(|v| if *v == 0 { None } else { Some(*v) })
                    .unwrap_or($default)
            }
        }

        if has_ignored_intermediates || intermediates.len() > 1 {
            return;
        }

        let terminal = &mut self.terminal;
        let writer = &mut self.writer;

        match (action, intermediates.get(0)) {
            // Insert blank lines
            ('@', None) => {
                terminal.insert_blank(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move the cursor up
            ('A', None) => {
                terminal.move_up(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move the cursor down
            ('B', None) | ('e', None) => {
                terminal.move_down(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move the cursor forward
            ('C', None) | ('a', None) => {
                terminal.move_forward(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move the cursor backwards
            ('D', None) => {
                terminal.move_backward(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move the cursor down and put a carriage return.
            ('E', None) => {
                terminal.move_down(get_arg!(idx: 0, def: 1) as usize);
                terminal.put_cr();
            },

            // Move the cursor up and put a carriage return.
            ('F', None) => {
                terminal.move_up(get_arg!(idx: 0, def: 1) as usize);
                terminal.put_cr();
            },

            // Go to column
            ('G', None) | ('`', None) => {
                terminal.goto_column(get_arg!(idx: 0, def: 1) as usize - 1);
            },

            // Set cursor position
            ('H', None) | ('f', None) => {
                terminal.goto(
                    get_arg!(idx: 0, def: 1) as usize - 1, // X
                    get_arg!(idx: 1, def: 1) as usize - 1, // Y
                );
            },

            // Clear screen
            ('J', None) => {
                let mode = match get_arg!(idx: 0, def: 0) {
                    0 => ClearScreenMode::Below,
                    1 => ClearScreenMode::Above,
                    2 => ClearScreenMode::All,
                    3 => ClearScreenMode::Saved,
                    _ => {
                        unhandled!();
                        return;
                    }
                };
                terminal.clear_screen(mode);
            },

            // Clear line
            ('K', None) => {
                let mode = match get_arg!(idx: 0, def: 0) {
                    0 => ClearLineMode::Right,
                    1 => ClearLineMode::Left,
                    2 => ClearLineMode::All,
                    _ => {
                        unhandled!();
                        return;
                    }
                };
                terminal.clear_line(mode);
            },

            // Insert blank lines
            ('L', None) => {
                terminal.insert_blank_lines(get_arg!(idx: 0, def: 1) as usize);
            },

            // Delete lines
            ('M', None) => {
                terminal.delete_lines(get_arg!(idx: 0, def: 1) as usize);
            },

            // Delete chars
            ('P', None) => {
                terminal.delete_chars(get_arg!(idx: 0, def: 1) as usize);
            },

            // Scroll up
            ('S', None) => {
                terminal.scroll_up(get_arg!(idx: 0, def: 1) as usize);
            },

            // Scroll down
            ('T', None) => {
                terminal.scroll_down(get_arg!(idx: 0, def: 1) as usize);
            },

            // Erase characters
            ('X', None) => {
                terminal.erase_chars(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move backward tab stops
            ('Z', None) => {
                terminal.move_backward_tabs(get_arg!(idx: 0, def: 1) as usize);
            },

            // Repeat last char
            ('b', None) => {
                if let Some(ch) = self.state.preceding_char {
                    for _ in 0..get_arg!(idx: 0, def: 1) {
                        terminal.put_char(ch);
                    }
                }
            },

            // Primary device attribute
            ('c', None) => {
                match get_arg!(idx: 0, def: 0) {
                    0 => {
                        write!(writer, "\x1b[?1;2c");
                    },
                    _ => unhandled!(),
                }
            },

            // Secondary device attribute
            ('c', Some(b'>')) => {
                match get_arg!(idx: 0, def: 0) {
                    0 => {
                        write!(writer, "\x1b[>84;0;0c");
                    },
                    _ => unhandled!(),
                }
            },

            // Go to line
            ('d', None) => {
                terminal.goto_line(get_arg!(idx: 0, def: 1) as usize - 1);
            },

            // Clear tabstops
            ('g', None) => {
                match get_arg!(idx: 0, def: 0) {
                    0 => {
                        terminal.unset_horizontal_tabstop(
                            terminal.cursor().x,
                        );
                    }
                    3 => terminal.unset_all_horizontal_tabstops(),
                    _ => {
                        unhandled!();
                        return;
                    }
                }
            },

            // Set mode
            ('h', intermediate) => {
                for arg in args {
                    match TerminalMode::from_primitive(intermediate, *arg) {
                        Some(mode) => terminal.set_mode(mode),
                        None => unhandled!(),
                    }
                }
            },

            // Unset mode
            ('l', intermediate) => {
                for arg in args {
                    match TerminalMode::from_primitive(intermediate, *arg) {
                        Some(mode) => terminal.unset_mode(mode),
                        None => unhandled!(),
                    }
                }
            },

            // Set SGR attribute
            ('m', None) => {
                if args.is_empty() {
                    terminal.sgr_attribute(sgr::Attribute::Reset);
                } else {
                    for attr in sgr::parse_attributes(&mut args.iter()) {
                        terminal.sgr_attribute(attr);
                    }
                }
            },
            // TODO tmux implements some complex behavior here. We must
            // first see if we need to implement it and then implement it
            // if required.
            // ('m', Some(b'>')) => {
                // let a = get_arg!(idx: 0, def: 0);
                // let b = get_arg!(idx: 1, def: 0);
                // match (a, b) {
                    // (0, _) | (4, 0) => {
                        // handler.unset_mode(TerminalMode::KExtended);
                    // },
                    // (4, 1) | (4, 2) => {
                        // handler.set_mode(TerminalMode::KExtended);
                    // },
                    // _ => unhandled!(),
                // }
            // },

            // Report device status
            ('n', None) => {
                match get_arg!(idx: 0, def: 0) {
                    5 => {
                        // respond with b"\x1b[0n"
                        write!(writer, "\x1b[0n");
                    },
                    6 => {
                        // respond with format!("\x1b[{};{}R", cursor_line, cursor_col)
                        let cur = terminal.cursor();
                        write!(writer, "\x1b[{};{}R", cur.y, cur.x);
                    },
                    _ => unhandled!(),
                }
            },
            // TODO same as ('m', Some(b'>'))
            // ('n', Some(b'>')) => {
                // match get_arg!(idx: 0, def: 0) {
                    // 4 => {
                        // handler.clear_mode(TerminalMode::KExtended);
                    // },
                    // _ => unhandled!(),
                // }
            // },

            // Set cursor style
            ('q', Some(b' ')) => {
                match CursorStyle::from_primitive(get_arg!(idx: 0, def: 0)) {
                    Some(style) => terminal.set_cursor_style(style),
                    None => unhandled!(),
                }
            },

            // Report program name and version
            // ('q', Some(b'>')) => {
                // match get_arg!(idx: 0, def: 0) {
                    // 0 => {
                        // respond with b"\x1bP>|{program_name} {version}\x1b\\"
                    // },
                    // _ => unhandled!(),
                // }
            // },

            // Set scrolling region
            ('r', None) => {
                let top = get_arg!(idx: 0, def: 1) as usize;
                let bottom = get_arg!(idx: 1, def: terminal.size().y as i64) as usize;
                terminal.set_scrolling_region(top, bottom);
            },

            // Save cursor position
            ('s', None) => {
                terminal.save_cursor_position();
            },

            // Save/restore title
            ('t', None) => {
                match get_arg!(idx: 0, def: 0) {
                    22 => terminal.save_title(),
                    23 => terminal.restore_title(),
                    _ => unhandled!(),
                }
            },

            // Restore cursor position
            ('u', None) => {
                terminal.restore_cursor_position();
            },

            _ => unhandled!(),
        }
    }

    fn put(&mut self, _byte: u8) {}

    fn hook(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _ch: char) {}

    fn unhook(&mut self) {}
}
