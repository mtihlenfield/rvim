use crate::char_iter;
use crate::gap_buf;
use crate::line_iter;
use crate::slice;

use crossterm::event;
use log::info;
use std::iter::Rev;

pub enum Mode {
    Normal,
    Insert,
}

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

    fn adjust_col(&self, line_len: usize) -> usize {
        if self.preffered_col >= line_len {
            line_len.saturating_sub(1)
        } else {
            self.preffered_col
        }
    }

    pub fn move_line(&mut self, line_index: usize, line_len: usize) {
        self.index = line_index + self.adjust_col(line_len);
    }
}

pub type BufferLines<'a> = line_iter::GapBufferLines<'a>;
pub type BufferChars<'a> = char_iter::GapBufferChars<'a>;
pub type BufferSlice<'a> = slice::GapBufferSlice<'a>;

#[derive(Debug)]
pub struct BufferError(String);

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

    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn insert(&mut self, c: char) {
        self.buf.insert(&c.to_string());

        // if c == '\n' {
        //     self.cursor.newline();
        // } else {
        //     self.cursor.right();
        // }
    }

    pub fn delete(&mut self) -> Result<(), BufferError> {
        match self.buf.delete() {
            Ok(()) => {
                //  TODO: move the cursor
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

    pub fn chars_at_rev(&'_ self, index: usize) -> Rev<BufferChars<'_>> {
        self.buf.chars_at_rev(index)
    }

    pub fn line_start(&self, index: usize) -> usize {
        self.buf.line_start(index)
    }

    pub fn line_end(&self, index: usize) -> usize {
        self.buf.line_end(index)
    }

    pub fn line_length(&self, start_index: usize) -> usize {
        self.buf.line_length(start_index)
    }

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

    pub fn find_next(&'_ self, start: usize, search_char: char) -> Option<usize> {
        self.buf.find_next(start, search_char)
    }

    pub fn move_right(&mut self) {
        if self.buf.get(self.cursor.index) == Some('\n') {
            // cursor is on an empty line
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
        if let Some(index) = self.buf.find_next(self.cursor.index, '\n') {
            if index == self.buf.len() - 1 {
                // Trailing newline
                return;
            }

            let line_len = self.buf.line_length(index + 1);
            self.cursor.move_line(index + 1, line_len);
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.index == 0 {
            return;
        }

        let Some(curr_line_start) = self.buf.find_prev(self.cursor.index - 1, '\n') else {
            return;
        };

        if let Some(prev_line_start) = self.buf.find_prev(curr_line_start - 1, '\n') {
            let prev_line_start = prev_line_start + 1;
            let line_len = curr_line_start.saturating_sub(prev_line_start + 1);
            self.cursor.move_line(prev_line_start, line_len);
        } else {
            self.cursor.move_line(0, self.buf.line_length(0));
        }
    }
}

pub struct EditorState {
    pub buffer: Buffer,
    pub mode: Mode,
}

impl EditorState {
    pub fn new() -> EditorState {
        EditorState {
            mode: Mode::Normal,
            buffer: Buffer::new(),
        }
    }

    pub fn open_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        self.buffer = Buffer::from_file(path)?;

        Ok(())
    }

    pub fn handle_normal_update(&mut self, key_ev: event::KeyEvent) -> bool {
        match key_ev.code {
            event::KeyCode::Char(c) => match c {
                'q' => true,
                'i' => {
                    self.mode = Mode::Insert;
                    info!("Switching to Insert mode.");
                    false
                }
                'l' => {
                    self.buffer.move_right();
                    false
                }
                'h' => {
                    self.buffer.move_left();
                    false
                }
                'j' => {
                    self.buffer.move_down();
                    false
                }
                'k' => {
                    self.buffer.move_up();
                    false
                }
                _ => false,
            },
            _ => false,
        }
    }

    pub fn handle_insert_update(&mut self, key_ev: event::KeyEvent) -> bool {
        match key_ev.code {
            event::KeyCode::Char(c) => {
                if !c.is_control() {
                    self.buffer.insert(c);
                }
            }
            event::KeyCode::Enter => {
                self.buffer.insert('\n');
            }
            event::KeyCode::Backspace => {
                if let Err(e) = self.buffer.delete() {
                    panic!("Failed to delete: {}", e.0);
                }
            }
            event::KeyCode::Esc => {
                // TODO: in vim this also results in a cursor change - it moves one char to the
                // left if it's at the end of a line (of text, not a row). Which means that if
                // you go straight back to insert mode you start one char to the left of where you
                // were before switching to normal mode.
                self.mode = Mode::Normal;
                info!("Switching to Normal mode.");
            }
            _ => {}
        };

        false
    }

    pub fn update(&mut self, key_ev: event::KeyEvent) -> bool {
        match self.mode {
            Mode::Normal => self.handle_normal_update(key_ev),
            Mode::Insert => self.handle_insert_update(key_ev),
        }
    }
}
