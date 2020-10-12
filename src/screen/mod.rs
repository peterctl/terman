pub mod renderer;

use {
    std::cmp::min,
    log::trace,
    crate::{
        ansi::{
            CharsetList,
            CharsetIndex,
            StandardCharset,
            Handler,
            CursorStyle,
            ClipboardType,
            ClearLineMode,
            ClearScreenMode,
            TerminalMode,
            sgr::Attribute,
            color::{
                SpecialColor,
                RgbColor,
            },
        },
        grid::{
            Grid,
            Cell,
            Attributes,
            Flags,
        },
        util::Point,
    }
};

pub struct Screen {
    grid: Grid,
    size: Point,
    cursor: Point,
    cell_template: Attributes,
    charsets: CharsetList,
    active_charset: CharsetIndex,
}

impl Screen {
    pub fn new(size: Point) -> Self {
        Self {
            size,
            grid: Grid::new(size),
            cursor: Point::default(),
            cell_template: Attributes::default(),
            charsets: CharsetList::default(),
            active_charset: CharsetIndex::default(),
        }
    }

    pub fn cursor_next(&mut self) {
        self.cursor.x += 1;
        if self.cursor.x >= self.size.x {
            self.cursor.x = 0;
            self.cursor.y += 1;
        }
    }

    pub fn cursor_prev(&mut self) {
        if self.cursor.x == 0 {
            self.cursor.x = self.size.x - 1;
            self.cursor.y -= 1;
        } else {
            self.cursor.x -= 1;
        }
    }

    pub fn cell<'a>(&'a self, point: Point) -> Option<&'a Cell> {
        self.grid.cell(point)
    }
}

impl Handler for Screen {
    fn size(&self) -> &Point {
        &self.size
    }

    fn cursor(&self) -> &Point {
        &self.cursor
    }

    fn put_char(&mut self, ch: char) {
        trace!("[handler] put_char: char={:?}", ch);
        let ch = self.charsets[self.active_charset].map(ch);
        let attributes = self.cell_template.clone();
        self.grid.cell_mut(self.cursor).map(|c| {
            c.ch = Some(ch);
            c.attributes = attributes;
        });
        self.cursor_next();
    }

    fn put_backspace(&mut self, count: usize) {
        trace!("[handler] put_backspace: count={:?}", count);
        for _ in 0..count {
            self.grid.cell_mut(self.cursor).map(|c| c.ch = None);
            self.cursor_prev();
        }
    }

    fn put_tab(&mut self, count: usize) {
        trace!("[handler] put_tab: count={:?}", count);
    }

    fn put_lf(&mut self) {
        trace!("[handler] put_lf");
        self.cursor.x = 0;
        self.cursor.y += 1;
    }

    fn put_cr(&mut self) {
        trace!("[handler] put_cr");
        self.cursor.x = 0;
    }

    fn insert_blank(&mut self, count: usize) {
        trace!("[handler] insert_blank: count={:?}", count);
    }

    fn insert_blank_lines(&mut self, count: usize) {
        trace!("[handler] insert_blank_lines: count={:?}", count);
    }

    fn clear_screen(&mut self, mode: ClearScreenMode) {
        trace!("[handler] clear_screen: mode={:?}", mode);
    }

    fn clear_line(&mut self, mode: ClearLineMode) {
        trace!("[handler] clear_line: mode={:?}", mode);
    }

    fn delete_lines(&mut self, count: usize) {
        trace!("[handler] delete_lines: count={:?}", count);
    }

    fn delete_chars(&mut self, count: usize) {
        trace!("[handler] delete_chars: count={:?}", count);
    }

    fn erase_chars(&mut self, count: usize) {
        trace!("[handler] erase_chars: count={:?}", count);
    }

    fn bell(&mut self) {
        trace!("[handler] bell");
    }

    fn do_alignment_test(&mut self) {
        trace!("[handler] do_alignment_test");
    }

    fn set_active_charset(&mut self, index: CharsetIndex) {
        trace!("[handler] set_active_charset: index={:?}", index);
        self.active_charset = index;
    }

    fn configure_charset(&mut self, index: CharsetIndex, charset: StandardCharset) {
        trace!("[handler] configure_charset: index={:?}, charset={:?}", index, charset);
        self.charsets[index] = charset;
    }

    fn set_title(&mut self, title: &str) {
        trace!("[handler] set_title: title={:?}", title);
    }

    fn save_title(&mut self) {
        trace!("[handler] save_title");
    }

    fn restore_title(&mut self) {
        trace!("[handler] restore_title");
    }

    fn set_path(&mut self, path: &str) {
        trace!("[handler] set_path: path={:?}", path);
    }

    fn get_clipboard(&mut self, clipboard: ClipboardType) -> Option<&Vec<u8>> {
        trace!("[handler] get_clipboard: clipboard={:?}", clipboard);
        None
    }

    fn set_clipboard(&mut self, clipboard: ClipboardType, data: &[u8]) {
        trace!(
            "[handler] set_clipboard: clipboard={:?}, data={:?}",
            clipboard,
            unsafe { std::str::from_utf8_unchecked(data) },
        );
    }

    fn save_cursor_position(&mut self) {
        trace!("[handler] save_cursor_position")
    }

    fn restore_cursor_position(&mut self) {
        trace!("[handler] restore_cursor_position")
    }

    fn set_cursor_style(&mut self, style: CursorStyle) {
        trace!("[handler] set_cursor_style: style={:?}", style);
    }

    fn set_horizontal_tabstop(&mut self, column: usize) {
        trace!("[handler] set_horizontal_tabstop: column={:?}", column);
    }

    fn unset_horizontal_tabstop(&mut self, column: usize) {
        trace!("[handler] unset_horizontal_tabstop: column={:?}", column);
    }

    fn unset_all_horizontal_tabstops(&mut self) {
        trace!("[handler] unset_all_horizontal_tabstops");
    }

    fn move_forward_tabs(&mut self, count: usize) {
        trace!("[handler] move_forward_tabs: count={:?}", count);
    }

    fn move_backward_tabs(&mut self, count: usize) {
        trace!("[handler] move_backward_tabs: count={:?}", count);
    }

    fn move_up(&mut self, count: usize) {
        trace!("[handler] move_up: count={:?}", count);
        if count > self.cursor.y {
            self.cursor.y = 0;
        } else {
            self.cursor.y -= count;
        }
    }

    fn move_down(&mut self, count: usize) {
        trace!("[handler] move_down: count={:?}", count);
        self.cursor.y = min(
            self.cursor.y + count,
            self.size.y - 1,
        );
    }

    fn move_forward(&mut self, count: usize) {
        trace!("[handler] move_forward: count={:?}", count);
        self.cursor.x = min(
            self.cursor.x + count,
            self.size.x - 1,
        );
    }

    fn move_backward(&mut self, count: usize) {
        trace!("[handler] move_backward: count={:?}", count);
        if count > self.size.x {
            self.size.x = 0;
        } else {
            self.size.x -= count;
        }
    }

    fn goto_column(&mut self, column: usize) {
        trace!("[handler] goto_column: column={:?}", column);
        self.cursor.x = column
    }

    fn goto_line(&mut self, line: usize) {
        trace!("[handler] goto_line: line={:?}", line);
        self.cursor.y = line
    }

    fn goto(&mut self, line: usize, column: usize) {
        trace!("[handler] goto: line={:?}, column={:?}", line, column);
        self.cursor.x = column;
        self.cursor.y = line;
    }

    fn scroll_up(&mut self, count: usize) {
        trace!("[handler] scroll_up: count={:?}", count);
    }

    fn scroll_down(&mut self, count: usize) {
        trace!("[handler] scroll_down: count={:?}", count);
    }

    fn set_scrolling_region(&mut self, top: usize, bottom: usize) {
        trace!("[handler] set_scrolling_region: top={:?}, bottom={:?}", top, bottom);
    }

    fn set_mode(&mut self, mode: TerminalMode) {
        trace!("[handler] set_mode: mode={:?}", mode);
    }

    fn unset_mode(&mut self, mode: TerminalMode) {
        trace!("[handler] unset_mode: mode={:?}", mode);
    }

    fn set_keypad_application_mode(&mut self) {
        trace!("[handler] set_keypad_application_mode");
    }

    fn unset_keypad_application_mode(&mut self) {
        trace!("[handler] unset_keypad_application_mode");
    }

    fn get_color(&mut self, index: u8) -> Option<&RgbColor> {
        trace!("[handler] get_color: index={:?}", index);
        None
    }

    fn set_color(&mut self, index: u8, color: RgbColor) {
        trace!("[handler] set_color: index={:?}, color={:?}", index, color);
    }

    fn reset_color(&mut self, index: u8) {
        trace!("[handler] reset_color: index={:?}", index);
    }

    fn reset_all_colors(&mut self) {
        trace!("[handler] reset_all_colors");
    }

    fn get_special_color(&mut self, index: SpecialColor) -> Option<&RgbColor> {
        trace!("[handler] get_special_color: index={:?}", index);
        None
    }

    fn set_special_color(&mut self, index: SpecialColor, color: RgbColor) {
        trace!("[handler] set_special_color: index={:?}, color={:?}", index, color);
    }

    fn reset_special_color(&mut self, index: SpecialColor) {
        trace!("[handler] reset_special_color: index={:?}", index);
    }

    fn sgr_attribute(&mut self, attr: Attribute) {
        trace!("[handler] sgr_attribute: attr={:?}", attr);
        match attr {
            Attribute::Reset => {
                self.cell_template = Attributes::default();
            },
            Attribute::Bold => {
                self.cell_template.flags.insert(Flags::BOLD);
            },
            Attribute::Dim => {
                self.cell_template.flags.insert(Flags::DIM);
            },
            Attribute::Italic => {
                self.cell_template.flags.insert(Flags::ITALIC);
            },
            Attribute::Underline => {
                self.cell_template.flags.insert(Flags::UNDERLINE);
            },
            Attribute::BlinkSlow => {
                self.cell_template.flags.insert(Flags::BLINK_SLOW);
                self.cell_template.flags.remove(Flags::BLINK_FAST);
            },
            Attribute::BlinkFast => {
                self.cell_template.flags.insert(Flags::BLINK_FAST);
                self.cell_template.flags.remove(Flags::BLINK_SLOW);
            },
            Attribute::Inverse => {
                self.cell_template.flags.insert(Flags::INVERSE);
            },
            Attribute::Hidden => {
                self.cell_template.flags.insert(Flags::HIDDEN);
            },
            Attribute::Strike => {
                self.cell_template.flags.insert(Flags::STRIKEOUT);
            },
            Attribute::CancelBold => {
                self.cell_template.flags.remove(Flags::BOLD);
            },
            Attribute::CancelBoldDim => {
                self.cell_template.flags.remove(Flags::DIM);
            },
            Attribute::CancelItalic => {
                self.cell_template.flags.remove(Flags::ITALIC);
            },
            Attribute::CancelUnderline => {
                self.cell_template.flags.remove(Flags::UNDERLINE);
            },
            Attribute::CancelBlink => {
                self.cell_template.flags.remove(Flags::BLINK_SLOW | Flags::BLINK_FAST);
            },
            Attribute::CancelInverse => {
                self.cell_template.flags.remove(Flags::INVERSE);
            },
            Attribute::CancelHidden => {
                self.cell_template.flags.remove(Flags::HIDDEN);
            },
            Attribute::CancelStrike => {
                self.cell_template.flags.remove(Flags::STRIKEOUT);
            },
            Attribute::Foreground(color) => {
                self.cell_template.fg = color;
            },
            Attribute::Background(color) => {
                self.cell_template.bg = color;
            },
        }
    }

    fn reverse_index(&mut self) {
        trace!("[handler] reverse_index");
    }

    fn reset_state(&mut self) {
        trace!("[handler] reset_state");
    }
}
