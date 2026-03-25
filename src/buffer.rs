use crate::char_iter;
use crate::gap_buf;
use crate::line_iter;
use crate::slice;
use std::iter::Rev;

#[derive(Debug, Clone)]
pub struct Cursor {
    pub index: usize,
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

pub struct Buffer {
    buf: gap_buf::GapBuffer,
    pub cursor: Cursor,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buf: gap_buf::GapBuffer::new(),
            cursor: Cursor::new(),
        }
    }

    pub fn from_file(path: &str) -> Result<Buffer, std::io::Error> {
        let s = std::fs::read_to_string(path)?;
        let buf = Buffer {
            buf: s.as_str().into(),
            cursor: Cursor::new(),
        };

        Ok(buf)
    }

    #[cfg(test)]
    pub fn from_string(s: &str) -> Buffer {
        Buffer {
            buf: s.into(),
            cursor: Cursor::new(),
        }
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

    #[cfg(test)]
    pub fn get(&self, index: usize) -> Option<char> {
        self.buf.get(index)
    }

    pub fn chars_at(&'_ self, index: usize) -> BufferChars<'_> {
        self.buf.chars_at(index)
    }

    // pub fn chars_at_rev(&'_ self, index: usize) -> Rev<BufferChars<'_>> {
    //     self.buf.chars_at_rev(index)
    // }

    pub fn line_start(&self, index: usize) -> usize {
        self.buf.line_start(index)
    }

    pub fn line_end(&self, index: usize) -> usize {
        self.buf.line_end(index)
    }

    // pub fn line_length(&self, start_index: usize) -> usize {
    //     self.buf.line_length(start_index)
    // }

    // pub fn lines_at(&'_ self, line_num: usize) -> Option<BufferLines<'_>> {
    //     self.buf.lines_at(line_num)
    // }

    // pub fn lines_at_char(&'_ self, index: usize) -> BufferLines<'_> {
    //     self.buf.lines_at_char(index)
    // }

    pub fn lines_at_char_rev(&'_ self, index: usize) -> Rev<BufferLines<'_>> {
        self.buf.lines_at_char_rev(index)
    }

    // pub fn lines_at_rev(&'_ self, line_num: usize) -> Option<Rev<BufferLines<'_>>> {
    //     self.buf.lines_at_rev(line_num)
    // }
    //

    // pub fn find_next(&'_ self, start: usize, search_char: char) -> Option<usize> {
    //     self.buf.find_next(start, search_char)
    // }

    pub fn move_right(&mut self, force: bool) {
        if self.buf.get(self.cursor.index) == Some('\n') || self.len() == 0 {
            // cursor is on an empty line
            return;
        }

        if force {
            // TODO: not working on the last line empty lines
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
        if self.cursor.index == 0 || self.buf.get(self.cursor.index - 1) == Some('\n') {
            return;
        }

        self.cursor.left();
    }

    pub fn move_down(&mut self) {
        if self.len() == 0 || self.cursor.index == self.len() {
            return;
        }

        if let Some(index) = self.buf.find_next(self.cursor.index, '\n') {
            if index == self.buf.len() - 1 {
                // Trailing newline
                return;
            }

            // TODO: move_line expects that line_len does not include the newline char
            let line_len = self.buf.line_length(index + 1);
            self.cursor.move_line(index + 1, line_len);
        }
    }

    pub fn move_up(&mut self) {
        // Some expected invariants:
        // - The cursor should only ever be on a \n if it is an empty line
        if self.cursor.index == 0 || self.len() == 0 {
            return;
        }

        let prev_line_end = match self.buf.find_prev(self.cursor.index, '\n') {
            // Note that the index == 0 guard above this means that we don't have to worry about
            // index being 0 here
            Some(e) if e == self.cursor.index => self.cursor.index - 1,
            Some(e) => e,
            None => return,
        };

        if let Some(pre_prev_line_end) = self.buf.find_prev(prev_line_end.saturating_sub(1), '\n') {
            let target_line_start = pre_prev_line_end + 1;
            let target_line_len = prev_line_end.saturating_sub(target_line_start);
            self.cursor.move_line(target_line_start, target_line_len);
        } else {
            // We're moving up to the first line, and we know that the line ends in a \n. Subtract
            // 1 to get the length without the \n
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
    }

    // TODO: tests with the cursor at one past the end of the buffer? Would only need to worry
    // about this during insert mode movements: left and right
}
