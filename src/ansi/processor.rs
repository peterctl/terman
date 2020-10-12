use std::str;
use log::trace;

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
    Handler,
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
        handler: &mut dyn Handler,
        writer: &mut dyn std::io::Write,
    ) {
        let mut performer = Performer::new(handler, writer, &mut self.state);
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
    handler: &'a mut dyn Handler,
    writer: &'a mut dyn std::io::Write,
    state: &'a mut ProcessorState,
}

impl<'a> Performer<'a> {
    pub fn new(
        handler: &'a mut dyn Handler,
        writer: &'a mut dyn std::io::Write,
        state: &'a mut ProcessorState,
    ) -> Self {
        Self {
            handler,
            writer,
            state,
        }
    }
}

impl<'a> ::vte::Perform for Performer<'a> {
    fn print(&mut self, ch: char) {
        trace!("[processor] print: char={:?}", ch);
        self.handler.put_char(ch);
        self.state.preceding_char = Some(ch);
    }

    fn execute(&mut self, byte: u8) {
        trace!("[processor] execute: byte={:?}", byte);
        match byte {
            C0::BEL => self.handler.bell(),
            C0::BS => self.handler.put_backspace(1),
            C0::HT => self.handler.put_tab(1),
            C0::LF | C0::VT | C0::FF => self.handler.put_lf(),
            C0::CR => self.handler.put_cr(),
            C0::SI => self.handler.set_active_charset(CharsetIndex::G0),
            C0::SO => self.handler.set_active_charset(CharsetIndex::G1),
            _ => {
                // TODO unhandled
            }
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        trace!("[processor] esc_dispatch: intermediates={:?}, ignore={:?}, byte={:?}", intermediates, _ignore, byte);
        match (byte, intermediates.get(0)) {
            (b'0', Some(b'(')) => self.handler.configure_charset(CharsetIndex::G0, StandardCharset::Ascii),
            (b'0', Some(b')')) => self.handler.configure_charset(CharsetIndex::G1, StandardCharset::Ascii),
            (b'7', None) => self.handler.save_cursor_position(),
            (b'8', None) => self.handler.restore_cursor_position(),
            (b'8', Some(b'#')) => self.handler.do_alignment_test(),
            (b'=', None) => self.handler.set_keypad_application_mode(),
            (b'>', None) => self.handler.unset_keypad_application_mode(),
            (b'B', Some(b'(')) => self.handler.configure_charset(CharsetIndex::G0, StandardCharset::Special),
            (b'B', Some(b')')) => self.handler.configure_charset(CharsetIndex::G1, StandardCharset::Special),
            (b'D', None) => self.handler.put_lf(),
            (b'E', None) => {
                self.handler.put_lf();
                self.handler.put_cr();
            },
            (b'H', None) => self.handler.set_horizontal_tabstop(
                self.handler.cursor().x,
            ),
            (b'M', None) => self.handler.reverse_index(),
            (b'c', None) => self.handler.reset_state(),
            (b'\\', None) => {},
            _ => {
                // TODO implement unhandled.
            }
        }
    }

    fn osc_dispatch(&mut self, params: &[&[u8]], bell_terminated: bool) {
        trace!("[processor] osc_dispatch: params={:?}, bell_terminated={:?}", params, bell_terminated);
        macro_rules! unhandled {
            () => {{
                // TODO
            }};
        }

        if params.is_empty() || params[0].is_empty() {
            unhandled!();
            return;
        }

        let handler = &mut self.handler;
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
                    handler.set_title(title.trim());
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
                    handler.set_color(index, color);
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

                handler.set_path(path.trim());
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
                            if let Some(ref color) = handler.get_special_color(index) {
                                let _ = write!(
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
                                handler.set_special_color(index, color);
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
                        handler.set_cursor_style(style);
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
                        if let Some(ref data) = handler.get_clipboard(clipboard_type) {
                            let data = base64::encode(data);
                            let _ = write!(
                                writer,
                                "\x1b]52;{};{}{}",
                                clipboard_char,
                                data,
                                terminator,
                            );
                        }
                    },
                    data => {
                        if let Ok(data) = base64::decode(data) {
                            handler.set_clipboard(clipboard_type, data.as_ref());
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
                        handler.reset_color(index);
                    }
                } else {
                    let index: Option<u8> = str::from_utf8(params[1]).ok()
                        .and_then(|s| s.parse().ok());
                    if let Some(index) = index {
                        handler.reset_color(index);
                    }
                }
            },

            // Reset foreground color
            110 => {
                handler.reset_special_color(SpecialColor::Foreground);
            },

            // Reset background color
            111 => {
                handler.reset_special_color(SpecialColor::Background);
            },

            // Reset cursor color
            112 => {
                handler.reset_special_color(SpecialColor::Cursor);
            },

            _ => unhandled!(),
        }
    }

    fn csi_dispatch(
        &mut self,
        args: &[i64],
        intermediates: &[u8],
        ignore_intermediates: bool,
        action: char,
    ) {
        trace!(
            "[processor] csi_dispatch: args={:?}, intermediates={:?}, ignore_intermediates={:?}, action={:?}",
            args,
            intermediates,
            ignore_intermediates,
            action,
        );
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

        if ignore_intermediates || intermediates.len() > 1 {
            return;
        }

        let handler = &mut self.handler;
        let writer = &mut self.writer;

        match (action, intermediates.get(0)) {
            // Insert blank lines
            ('@', None) => {
                handler.insert_blank(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move the cursor up
            ('A', None) => {
                handler.move_up(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move the cursor down
            ('B', None) | ('e', None) => {
                handler.move_down(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move the cursor forward
            ('C', None) | ('a', None) => {
                handler.move_forward(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move the cursor backwards
            ('D', None) => {
                handler.move_backward(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move the cursor down and put a carriage return.
            ('E', None) => {
                handler.move_down(get_arg!(idx: 0, def: 1) as usize);
                handler.put_cr();
            },

            // Move the cursor up and put a carriage return.
            ('F', None) => {
                handler.move_up(get_arg!(idx: 0, def: 1) as usize);
                handler.put_cr();
            },

            // Go to column
            ('G', None) | ('`', None) => {
                handler.goto_column(get_arg!(idx: 0, def: 1) as usize - 1);
            },

            // Set cursor position
            ('H', None) | ('f', None) => {
                handler.goto(
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
                handler.clear_screen(mode);
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
                handler.clear_line(mode);
            },

            // Insert blank lines
            ('L', None) => {
                handler.insert_blank_lines(get_arg!(idx: 0, def: 1) as usize);
            },

            // Delete lines
            ('M', None) => {
                handler.delete_lines(get_arg!(idx: 0, def: 1) as usize);
            },

            // Delete chars
            ('P', None) => {
                handler.delete_chars(get_arg!(idx: 0, def: 1) as usize);
            },

            // Scroll up
            ('S', None) => {
                handler.scroll_up(get_arg!(idx: 0, def: 1) as usize);
            },

            // Scroll down
            ('T', None) => {
                handler.scroll_down(get_arg!(idx: 0, def: 1) as usize);
            },

            // Erase characters
            ('X', None) => {
                handler.erase_chars(get_arg!(idx: 0, def: 1) as usize);
            },

            // Move backward tab stops
            ('Z', None) => {
                handler.move_backward_tabs(get_arg!(idx: 0, def: 1) as usize);
            },

            // Repeat last char
            ('b', None) => {
                if let Some(ch) = self.state.preceding_char {
                    for _ in 0..get_arg!(idx: 0, def: 1) {
                        handler.put_char(ch);
                    }
                }
            },

            // Primary device attribute
            ('c', None) => {
                match get_arg!(idx: 0, def: 0) {
                    0 => {
                        let _ = write!(writer, "\x1b[?1;2c");
                    },
                    _ => unhandled!(),
                }
            },

            // Secondary device attribute
            ('c', Some(b'>')) => {
                match get_arg!(idx: 0, def: 0) {
                    0 => {
                        let _ = write!(writer, "\x1b[>84;0;0c");
                    },
                    _ => unhandled!(),
                }
            },

            // Go to line
            ('d', None) => {
                handler.goto_line(get_arg!(idx: 0, def: 1) as usize - 1);
            },

            // Clear tabstops
            ('g', None) => {
                match get_arg!(idx: 0, def: 0) {
                    0 => {
                        handler.unset_horizontal_tabstop(
                            handler.cursor().x,
                        );
                    }
                    3 => handler.unset_all_horizontal_tabstops(),
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
                        Some(mode) => handler.set_mode(mode),
                        None => unhandled!(),
                    }
                }
            },

            // Unset mode
            ('l', intermediate) => {
                for arg in args {
                    match TerminalMode::from_primitive(intermediate, *arg) {
                        Some(mode) => handler.unset_mode(mode),
                        None => unhandled!(),
                    }
                }
            },

            // Set SGR attribute
            ('m', None) => {
                if args.is_empty() {
                    handler.sgr_attribute(sgr::Attribute::Reset);
                } else {
                    for attr in sgr::parse_attributes(&mut args.iter()) {
                        handler.sgr_attribute(attr);
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
                        let _ = write!(writer, "\x1b[0n");
                    },
                    6 => {
                        // respond with format!("\x1b[{};{}R", cursor_line, cursor_col)
                        let cur = handler.cursor();
                        let _ = write!(writer, "\x1b[{};{}R", cur.y, cur.x);
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
                    Some(style) => handler.set_cursor_style(style),
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
                let bottom = get_arg!(idx: 1, def: handler.size().y as i64) as usize;
                handler.set_scrolling_region(top, bottom);
            },

            // Save cursor position
            ('s', None) => {
                handler.save_cursor_position();
            },

            // Save/restore title
            ('t', None) => {
                match get_arg!(idx: 0, def: 0) {
                    22 => handler.save_title(),
                    23 => handler.restore_title(),
                    _ => unhandled!(),
                }
            },

            // Restore cursor position
            ('u', None) => {
                handler.restore_cursor_position();
            },

            _ => unhandled!(),
        }
    }

    fn put(&mut self, _byte: u8) {}

    fn hook(&mut self, _params: &[i64], _intermediates: &[u8], _ignore: bool, _ch: char) {}

    fn unhook(&mut self) {}
}
