use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};

#[derive(Debug, Default, Eq, PartialEq)]
pub struct InputState {
    /// The input buffer.
    buffer: String,
    /// Position of the cursor based on characters (not bytes) in the text area.
    cursor_pos: usize,
}

impl InputState {
    /// Obtains a reference to the input buffer's contents.
    pub fn text(&self) -> &str {
        &*self.buffer
    }

    /// Moves the cursor to the start of the input.
    pub fn reset_cursor(&mut self) {
        self.cursor_pos = 0;
    }

    /// Moves the cursor to the end of the input.
    pub fn end_cursor(&mut self) {
        self.cursor_pos = self.buffer.chars().count();
    }

    /// Moves the cursor left by one character in the input.
    pub fn move_cursor_left(&mut self) {
        self.cursor_pos = self
            .cursor_pos
            .saturating_sub(1)
            .clamp(0, self.buffer.chars().count());
    }

    /// Moves the cursor right by one character in the input.
    pub fn move_cursor_right(&mut self) {
        self.cursor_pos = self
            .cursor_pos
            .saturating_add(1)
            .clamp(0, self.buffer.chars().count());
    }

    /// Moves the cursor left by one character in the input.
    pub fn add_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.buffer.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Deletes one previous character from the current cursor position in the input.
    pub fn delete_char(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }

        // Method "remove" is not used on the saved text for deleting the selected char.
        // Reason: Using remove on String works on bytes instead of the chars.
        // Using remove would require special care because of char boundaries.

        let current_index = self.cursor_pos;
        let from_left_to_current_index = current_index - 1;

        // Getting all characters before the selected character.
        let before_char_to_delete = self.buffer.chars().take(from_left_to_current_index);
        // Getting all characters after selected character.
        let after_char_to_delete = self.buffer.chars().skip(current_index);

        // Put all characters together except the selected one.
        // By leaving the selected one out, it is forgotten and therefore deleted.
        self.buffer = before_char_to_delete.chain(after_char_to_delete).collect();
        self.move_cursor_left();
    }

    /// Handles a key event
    pub fn handle_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char(char)
                if matches!(event.kind, KeyEventKind::Press | KeyEventKind::Repeat) =>
            {
                self.add_char(char)
            }
            KeyCode::Backspace => self.delete_char(),
            KeyCode::Left => self.move_cursor_left(),
            KeyCode::Right => self.move_cursor_right(),
            KeyCode::Home => self.reset_cursor(),
            KeyCode::End => self.end_cursor(),
            _ => {}
        }
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.buffer
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.cursor_pos)
            .unwrap_or(self.buffer.len())
    }
}
