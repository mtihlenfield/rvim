use crate::char_iter;
use crate::gap_buf;
use crate::line_iter;
use crate::slice;
use std::io::Write;
use std::iter::Rev;

#[derive(Debug, Clone)]
pub struct Cursor {
    index: usize,
    preffered_col: usize,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            index: 0,
            preffered_col: 0,
        }
    }

    pub fn right(&mut self) {
        self.index += 1;
        self.preffered_col += 1;
    }

    pub fn left(&mut self) {
        self.index -= 1;
        self.preffered_col -= 1;
    }

    /// NOTE: line_len should not include the \n char
    fn adjust_col(&self, line_len: usize) -> usize {
        if self.preffered_col >= line_len {
            // -1 because this is a length, not an index
            line_len.saturating_sub(1)
        } else {
            self.preffered_col
        }
    }

    /// NOTE: line_len should not include the \n char
    pub fn move_line(&mut self, line_index: usize, line_len: usize) {
        self.index = line_index + self.adjust_col(line_len);
    }

    #[cfg(test)]
    pub fn jump(&mut self, index: usize, preferred_col: usize) {
        self.index = index;
        self.preffered_col = preferred_col;
    }
}

pub type BufferLines<'a> = line_iter::GapBufferLines<'a>;
pub type BufferChars<'a> = char_iter::GapBufferChars<'a>;
pub type BufferSlice<'a> = slice::GapBufferSlice<'a>;

#[derive(Debug)]
pub struct BufferError(pub String);

#[derive(Debug)]
pub enum SaveError {
    NoFileName,
    IOError(std::io::Error),
}

impl std::fmt::Display for SaveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SaveError::NoFileName => write!(f, "No file name."),
            SaveError::IOError(e) => write!(f, "{}", e.to_string()),
        };

        Ok(())
    }
}

impl From<std::io::Error> for SaveError {
    fn from(err: std::io::Error) -> SaveError {
        SaveError::IOError(err)
    }
}

pub struct Buffer {
    buf: gap_buf::GapBuffer,
    cursor: Cursor,
    path: Option<String>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buf: gap_buf::GapBuffer::new(),
            cursor: Cursor::new(),
            path: None,
        }
    }

    pub fn from_file(path: &str) -> Result<Buffer, std::io::Error> {
        let s = std::fs::read_to_string(path)?;
        let buf = Buffer {
            buf: s.as_str().into(),
            cursor: Cursor::new(),
            path: Some(path.to_string()),
        };

        Ok(buf)
    }

    #[cfg(test)]
    pub fn from_string(s: &str) -> Buffer {
        Buffer {
            buf: s.into(),
            cursor: Cursor::new(),
            path: None,
        }
    }

    pub fn save(&mut self, path: Option<&str>) -> Result<(), SaveError> {
        let output_path = if let Some(p) = path {
            self.path = Some(p.to_string());
            p
        } else if let Some(p) = &self.path {
            p
        } else {
            return Err(SaveError::NoFileName);
        };

        let mut file = std::fs::File::create(output_path)?;
        write!(file, "{}", self.buf)?;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn insert(&mut self, c: char) {
        self.buf
            .insert_at(&c.to_string(), self.cursor.index)
            .expect("Failed to insert char");
        self.cursor.right();
    }

    pub fn delete(&mut self) -> Result<(), BufferError> {
        match self.buf.delete_at(self.cursor.index) {
            Ok(()) => {
                self.cursor.left();
                Ok(())
            }
            Err(gap_buf::GapBufferError::DeleteFromStart) => Ok(()),
            Err(e) => Err(BufferError(e.to_string())),
        }
    }

    pub fn get(&self, index: usize) -> Option<char> {
        self.buf.get(index)
    }

    pub fn chars_at(&'_ self, index: usize) -> BufferChars<'_> {
        self.buf.chars_at(index)
    }

    pub fn line_start(&self, index: usize) -> usize {
        self.buf.line_start(index)
    }

    pub fn line_end(&self, index: usize) -> usize {
        self.buf.line_end(index)
    }

    pub fn lines_at_char_rev(&'_ self, index: usize) -> Rev<BufferLines<'_>> {
        self.buf.lines_at_char_rev(index)
    }

    pub fn cursor_index(&self) -> usize {
        return self.cursor.index;
    }

    pub fn move_right(&mut self, append_mode: bool) {
        // Some expected invariants:
        // - The cursor *can* be on a newline, because we move left/right in insert mode
        // - The cursor cursor *can* be == buf.len()
        if self.buf.get(self.cursor.index) == Some('\n') || self.len() == 0 {
            // cursor is on an empty line
            return;
        }

        // Append mode is different in that:
        // - It can put the cursor on a newline even if the line is not empty
        // - It can put the cursor at buf.len() (an invalid index)
        // This means that when in append mode we always move the cursor, no matter
        // where is is at
        if append_mode {
            self.cursor.right();
            return;
        }

        if let Some(ch) = self.buf.get(self.cursor.index + 1)
            && ch != '\n'
        {
            self.cursor.right();
        }
    }

    pub fn move_left(&mut self) {
        // Some expected invariants:
        // - The cursor *can* be on a newline, because we move left/right in insert mode
        // - The cursor cursor *can* be == buf.len()
        if self.cursor.index == 0 {
            return;
        }

        if self.cursor.index == self.buf.len() {
            // We're transition from insert to normal mode and the cursor is invalid (by 1), so we have to
            // decrease it no matter what.
            self.cursor.left();
            return;
        }

        if self.buf.get(self.cursor.index - 1) == Some('\n') {
            return;
        }

        // There is one sort of odd case we are covering here: Apparently it is a posix standard that all
        // text files end in a \n. Text editors do not usually *show* this line, so you shouldn't
        // have your cursor on it visually, but you could have your cursor on it in append mode.
        // And in that case you just want to move left.
        self.cursor.left();
    }

    pub fn move_down(&mut self) {
        // Some expected invariants:
        // - The cursor should only ever be on a \n if it is an empty line
        // - The cursor will never be >= buf.len()
        if self.len() == 0 || self.cursor.index == self.len() {
            return;
        }

        if let Some(index) = self.buf.find_next(self.cursor.index, '\n') {
            if index == self.buf.len() - 1 {
                // Trailing newline
                return;
            }

            let line_start = index + 1;

            // We need to find the line length of the next line, but we need to account
            // for the fact that line_start could:
            // - point to the start of a normal line that ends in a \n
            // - point to a \n, indicating that the next line is empty
            // - point to the last line in the file, which does not end in a \n
            let mut line_len = self.buf.line_length(line_start);
            // note that we know line_len can't be 0. The only way that is possible is if
            // index points to a trailing newline at the end of the file, and we've already
            // returned if that was the case.
            line_len = if self.buf.get(line_start + line_len - 1) == Some('\n') {
                line_len - 1
            } else {
                line_len
            };

            self.cursor.move_line(line_start, line_len);
        }
    }

    pub fn move_up(&mut self) {
        // Some expected invariants:
        // - The cursor should only ever be on a \n if it is an empty line
        // - The cursor will never be >= buf.len()
        if self.cursor.index == 0 || self.len() == 0 {
            return;
        }

        let prev_line_end = match self.buf.find_prev(self.cursor.index, '\n') {
            // Note that the index == 0 guard above this means that we don't have to worry about
            // index being 0 here
            Some(e) if e == self.cursor.index => self.cursor.index - 1,
            Some(e) => e,
            // There is no newline before the cursor, which means we are on the very first line and
            // can't move up
            None => return,
        };

        if prev_line_end == 0 {
            // The line above is the first line, and it is just a single `\n` char
            self.cursor.move_line(0, 0);
            return;
        }

        if let Some(pre_prev_line_end) = self.buf.find_prev(prev_line_end.saturating_sub(1), '\n') {
            let target_line_start = pre_prev_line_end + 1;
            let target_line_len = prev_line_end.saturating_sub(target_line_start);
            self.cursor.move_line(target_line_start, target_line_len);
        } else {
            // We're moving up to the first line, the line is not empty, and we know that the line ends in a \n.
            // Subtract 1 to get the length without the \n
            self.cursor.move_line(0, self.buf.line_length(0) - 1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! assert_cursor {
        ($buff:expr, $pos:expr, $ch:expr) => {
            assert_eq!($buff.cursor.index, $pos);
            assert_eq!($buff.get($buff.cursor.index), Some($ch));
        };
    }

    #[test]
    fn test_move_empty_buffer() {
        let mut buff = Buffer::new();
        buff.move_up();
        assert_eq!(buff.cursor.index, 0);

        buff.move_down();
        assert_eq!(buff.cursor.index, 0);

        buff.move_left();
        assert_eq!(buff.cursor.index, 0);

        buff.move_right(false);
        assert_eq!(buff.cursor.index, 0);

        buff.move_right(true);
        assert_eq!(buff.cursor.index, 0);
    }

    // --- move up tests ---

    #[test]
    fn test_move_up_one_line() {
        let mut buff = Buffer::from_string("hello");

        // With cursor at first char in buffer
        buff.move_up();
        assert_cursor!(&buff, 0, 'h');

        // With cursor at random char in line
        buff.cursor.jump(3, 0);
        buff.move_up();
        assert_cursor!(&buff, 3, 'l');
    }

    #[test]
    fn test_move_up_two_lines() {
        let mut buff = Buffer::from_string("hello\ngoodbye");

        // With cursor at first char on second line
        buff.cursor.index = 6;
        buff.move_up();
        assert_cursor!(&buff, 0, 'h');

        // With cursor at char in middle of second line
        buff.cursor.jump(10, 4);
        buff.move_up();
        assert_cursor!(&buff, 4, 'o');

        // With cursor at last char in second line
        buff.cursor.jump(12, 6);
        buff.move_up();
        // Should end at the last char in the first line because preferred_col is longer
        // than the line length
        assert_cursor!(&buff, 4, 'o');
    }

    #[test]
    fn test_move_up_normal() {
        let mut buff = Buffer::from_string("hello\nworld\ngoodbye");

        // With cursor at first char on third line
        buff.cursor.jump(12, 0);
        buff.move_up();
        // Should end at the first char in the second line
        assert_cursor!(&buff, 6, 'w');

        // with cursor at char in middle of 3rd line
        buff.cursor.jump(15, 3);
        buff.move_up();
        assert_cursor!(&buff, 9, 'l');

        // With cursor at last char in third line
        buff.cursor.jump(18, 6);
        buff.move_up();
        // Should end at the last char in the second line because preferred_col is longer
        // than the line length
        assert_cursor!(&buff, 10, 'd');
    }

    #[test]
    fn test_move_up_empty_line() {
        let mut buff = Buffer::from_string("\nhello\n\nworld\n");

        // With cursor on the empty line at the end
        buff.cursor.jump(13, 0);
        buff.move_up();
        assert_cursor!(&buff, 8, 'w');

        // With cursor on the empty line in the middle
        buff.cursor.jump(6, 0);
        buff.move_up();
        assert_cursor!(&buff, 1, 'h');

        // With cursor on the empty line at the start
        buff.cursor.jump(0, 0);
        buff.move_up();
        assert_cursor!(&buff, 0, '\n');

        let mut buff = Buffer::from_string("\n\n");
        buff.cursor.jump(1, 0);
        buff.move_up();
        assert_cursor!(&buff, 0, '\n');
    }

    // --- move down tests ---

    #[test]
    fn test_move_down_one_line() {
        let mut buff = Buffer::from_string("hello");

        // With cursor at first char in buffer
        buff.move_down();
        assert_cursor!(&buff, 0, 'h');

        // With cursor at random char in line
        buff.cursor.jump(3, 0);
        buff.move_down();
        assert_cursor!(&buff, 3, 'l');
    }

    #[test]
    fn test_move_down_two_lines() {
        let mut buff = Buffer::from_string("goodbye\nhello");

        // With cursor at first char on first line
        buff.cursor.jump(0, 0);
        buff.move_down();
        assert_cursor!(&buff, 8, 'h');

        // With cursor at char in middle of the first line
        buff.cursor.jump(3, 3);
        buff.move_down();
        assert_cursor!(&buff, 11, 'l');

        // With cursor at last char in the first line
        buff.cursor.jump(6, 6);
        buff.move_down();
        // Should end at the last char in the second line because preferred_col is longer
        // than the line length
        assert_cursor!(&buff, 12, 'o');
    }

    #[test]
    fn test_move_down_normal() {
        let mut buff = Buffer::from_string("goodbye\nhello\nworld");
        // With cursor at first char on first line
        buff.cursor.jump(0, 0);
        buff.move_down();
        // Should end at the first char in the second line
        assert_cursor!(&buff, 8, 'h');

        // with cursor at char in middle of 3rd line
        buff.cursor.jump(3, 3);
        buff.move_down();
        assert_cursor!(&buff, 11, 'l');

        // With cursor at last char in third line
        buff.cursor.jump(6, 6);
        buff.move_down();
        // Should end at the last char in the second line because preferred_col is longer
        // than the line length
        assert_cursor!(&buff, 12, 'o');
    }

    #[test]
    fn test_move_down_empty_line() {
        let mut buff = Buffer::from_string("\nhello\n\nworld\n");

        // With cursor on the empty line at the start
        buff.cursor.jump(0, 0);
        buff.move_down();
        assert_cursor!(&buff, 1, 'h');

        // With cursor at first full line, moving to empty line
        buff.cursor.jump(3, 2);
        buff.move_down();
        assert_cursor!(&buff, 7, '\n');

        // With cursor on the empty line in the middle
        buff.cursor.jump(7, 0);
        buff.move_down();
        assert_cursor!(&buff, 8, 'w');

        // With cursor at second to last (full) line
        buff.cursor.jump(8, 0);
        buff.move_down();
        assert_cursor!(&buff, 8, 'w');

        let mut buff = Buffer::from_string("\n\n");
        buff.cursor.jump(0, 0);
        buff.move_down();
        assert_cursor!(&buff, 1, '\n');
    }

    // --- move right tests ---

    #[test]
    fn test_move_right_normal() {
        let mut buff = Buffer::from_string("hello");
        buff.cursor.jump(2, 2);
        buff.move_right(false);
        assert_cursor!(&buff, 3, 'l');
        assert_eq!(buff.cursor.preffered_col, 3);
    }

    #[test]
    fn test_move_right_end_of_line() {
        let mut buff = Buffer::from_string("hello\nworld");
        buff.cursor.jump(4, 4);
        buff.move_right(false);
        assert_cursor!(&buff, 4, 'o');
        assert_eq!(buff.cursor.preffered_col, 4);

        buff.move_right(true);
        assert_cursor!(&buff, 5, '\n');
        assert_eq!(buff.cursor.preffered_col, 5);
    }

    #[test]
    fn test_move_right_empty_line() {
        let mut buff = Buffer::from_string("hello\n\nworld");
        buff.cursor.jump(6, 0);
        buff.move_right(false);
        assert_cursor!(&buff, 6, '\n');
        assert_eq!(buff.cursor.preffered_col, 0);

        // if you enter append mode on an empty line, the cursor should stay at the same spot
        buff.move_right(true);
        assert_cursor!(&buff, 6, '\n');
        assert_eq!(buff.cursor.preffered_col, 0);
    }

    #[test]
    fn test_move_right_eof() {
        let mut buff = Buffer::from_string("hello");
        buff.cursor.jump(4, 4);
        buff.move_right(false);
        assert_cursor!(&buff, 4, 'o');
        assert_eq!(buff.cursor.preffered_col, 4);

        // if you enter append mode with the cursor at the very end of the buffer, the cursor
        // should move one past the end (an invalid index)
        buff.move_right(true);
        assert_eq!(buff.cursor.index, 5);
        assert_eq!(buff.get(buff.cursor.index), None);
        assert_eq!(buff.cursor.preffered_col, 5);
    }

    // --- move left tests ---

    #[test]
    fn test_move_left_normal() {
        let mut buff = Buffer::from_string("hello");
        buff.cursor.jump(2, 2);
        buff.move_left();
        assert_cursor!(&buff, 1, 'e');
        assert_eq!(buff.cursor.preffered_col, 1);
    }

    #[test]
    fn test_move_left_start_of_line() {
        let mut buff = Buffer::from_string("hello\nworld");
        buff.cursor.jump(6, 0);
        buff.move_left();
        assert_cursor!(&buff, 6, 'w');
        assert_eq!(buff.cursor.preffered_col, 0);

        buff.cursor.jump(0, 0);
        buff.move_left();
        assert_cursor!(&buff, 0, 'h');
        assert_eq!(buff.cursor.preffered_col, 0);
    }

    #[test]
    fn test_move_left_end_of_line() {
        // test moving left with cursor that is on a \n that is not an empty line
        let mut buff = Buffer::from_string("hello\nworld\ngoodbye\n");
        buff.cursor.jump(11, 5);
        buff.move_left();
        assert_cursor!(&buff, 10, 'd');
        assert_eq!(buff.cursor.preffered_col, 4);

        // Same thing as above, but with a trailing new line.
        buff.cursor.jump(19, 7);
        buff.move_left();
        assert_cursor!(&buff, 18, 'e');
        assert_eq!(buff.cursor.preffered_col, 6);
    }

    #[test]
    fn test_move_left_empty_line() {
        let mut buff = Buffer::from_string("hello\n\nworld");
        buff.cursor.jump(6, 0);
        buff.move_left();
        assert_cursor!(&buff, 6, '\n');
        assert_eq!(buff.cursor.preffered_col, 0);
    }

    #[test]
    fn test_move_left_eof() {
        let mut buff = Buffer::from_string("hello");
        // The cursor can end up past the end of the buffer if we are in insert (append) mode.
        buff.cursor.jump(5, 5);
        buff.move_left();
        assert_cursor!(&buff, 4, 'o');
        assert_eq!(buff.cursor.preffered_col, 4);
    }
}
