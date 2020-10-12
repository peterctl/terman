use crate::ansi::{
    charset::{
        CharsetIndex,
        StandardCharset,
    },
    color::{
        SpecialColor,
        RgbColor,
    },
    sgr::Attribute,
};
use crate::util::Point;
use super::{
    CursorStyle,
    ClipboardType,
    ClearLineMode,
    ClearScreenMode,
    TerminalMode,
};

/// Trait representing the actions that a terminal can receive.
pub trait Handler {
    /// Get the terminal size.
    fn size(&self) -> &Point;

    /// Get the current cursor position.
    fn cursor(&self) -> &Point;

    /// Print a character to the screen.
    fn put_char(&mut self, ch: char);

    /// Print a backspace to the screen.
    fn put_backspace(&mut self, amount: usize);

    /// Print `amount` tabs to the screen.
    fn put_tab(&mut self, amount: usize);

    /// Print a line feed to the screen.
    fn put_lf(&mut self);

    /// Print a carriage return to the screen.
    fn put_cr(&mut self);

    /// Insert `count` blank spaces.
    fn insert_blank(&mut self, count: usize);

    /// Insert `count` blank lines.
    fn insert_blank_lines(&mut self, count: usize);

    /// Clear the screen.
    fn clear_screen(&mut self, mode: ClearScreenMode);

    /// Clear the current line.
    fn clear_line(&mut self, mode: ClearLineMode);

    /// Delete `count` lines at and below the cursor.
    fn delete_lines(&mut self, count: usize);

    /// Delete `count` chars starting from the cursor.
    /// This removes the deleted cells and moves the remaining cells backwards.
    fn delete_chars(&mut self, count: usize);

    /// Erase `count` chars starting from the cursor.
    /// This replaces the erased characters with blank cells.
    fn erase_chars(&mut self, count: usize);

    /// Trigger the terminal bell.
    fn bell(&mut self);

    /// Do an alignment test.
    fn do_alignment_test(&mut self);

    /// Set the active charset.
    fn set_active_charset(&mut self, index: CharsetIndex);

    /// Configure the active charset.
    fn configure_charset(&mut self, index: CharsetIndex, charset: StandardCharset);

    /// Set the window title.
    fn set_title(&mut self, title: &str);

    /// Save title to stack.
    fn save_title(&mut self);

    /// Pop and restore title from stack.
    fn restore_title(&mut self);

    /// Set the current working directory for the running process.
    fn set_path(&mut self, path: &str);

    /// Load data from the clipboard.
    fn get_clipboard(&mut self, clipboard: ClipboardType) -> Option<&Vec<u8>>;

    /// Store data to the clipboard.
    fn set_clipboard(&mut self, clipboard: ClipboardType, data: &[u8]);

    /// Save the current cursor position.
    fn save_cursor_position(&mut self);

    /// Restore the last saved cursor position.
    fn restore_cursor_position(&mut self);

    /// Set the cursor style.
    fn set_cursor_style(&mut self, style: CursorStyle);

    /// Set `column` as a tabstop.
    fn set_horizontal_tabstop(&mut self, column: usize);

    /// Unset `column` as a tabstop.
    fn unset_horizontal_tabstop(&mut self, column: usize);

    /// Unset all tabstops.
    fn unset_all_horizontal_tabstops(&mut self);

    /// Move forward `count` tab stops.
    fn move_forward_tabs(&mut self, count: usize);

    /// Move backward `count` tab stops.
    fn move_backward_tabs(&mut self, count: usize);

    /// Move the cursor up `count` lines.
    fn move_up(&mut self, count: usize);

    /// Move the cursor down `count` lines.
    fn move_down(&mut self, count: usize);

    /// Move forward `count` columns.
    fn move_forward(&mut self, count: usize);

    /// Move backward `count` columns.
    fn move_backward(&mut self, count: usize);

    /// Go to the given column.
    fn goto_column(&mut self, column: usize);

    /// Go to the given line.
    fn goto_line(&mut self, line: usize);

    /// Go to the given line and column.
    fn goto(&mut self, line: usize, column: usize);

    /// Scroll up `count` lines.
    fn scroll_up(&mut self, count: usize);

    /// Scroll down `count` lines.
    fn scroll_down(&mut self, count: usize);

    /// Set scrolling region.
    fn set_scrolling_region(&mut self, top: usize, bottom: usize);

    /// Set terminal mode.
    fn set_mode(&mut self, mode: TerminalMode);

    /// Unset terminal mode.
    fn unset_mode(&mut self, mode: TerminalMode);

    fn set_keypad_application_mode(&mut self);

    fn unset_keypad_application_mode(&mut self);

    /// Set the color for an index.
    fn set_color(&mut self, index: u8, color: RgbColor);

    /// Get the spec for a color.
    fn get_color(&mut self, index: u8) -> Option<&RgbColor>;

    /// Reset indexed color.
    fn reset_color(&mut self, index: u8);

    /// Reset all indexed colors.
    fn reset_all_colors(&mut self);

    /// Set a special color.
    fn set_special_color(&mut self, index: SpecialColor, color: RgbColor);

    /// Get the spec for a special color.
    fn get_special_color(&mut self, index: SpecialColor) -> Option<&RgbColor>;

    /// Reset a special color.
    fn reset_special_color(&mut self, index: SpecialColor);

    /// Set SGR attribute.
    fn sgr_attribute(&mut self, attr: Attribute);

    /// Reverse Index.
    ///
    /// Move the active position to the same horizontal position on the
    /// preceding line. If the active position is at the top margin, a scroll
    /// down is performed.
    fn reverse_index(&mut self);

    /// Reset the terminal state.
    fn reset_state(&mut self);
}
