use crate::gap_buf;
use crate::position::Position;
use crossterm::event;
use log::info;

pub enum Mode {
    Normal,
    Insert,
    CommandLine,
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pos: Position,
    preffered_col: usize,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            // The position relative to the start of the buffer contents
            pos: Position::new(),
            preffered_col: 0,
        }
    }

    pub fn newline(&mut self) {
        self.pos.newline();
    }

    pub fn right(&mut self) {
        self.pos.right();
        self.preffered_col += 1;
    }

    pub fn left(&mut self) {
        self.pos.left();
        self.preffered_col -= 1;
    }

    fn adjust_col(&self, line_len: usize) -> usize {
        if line_len == 0 {
            0
        } else if self.preffered_col >= line_len {
            line_len - 1
        } else {
            self.preffered_col
        }
    }

    pub fn up(&mut self, line_len: usize) {
        self.pos.up(self.adjust_col(line_len));
    }

    pub fn down(&mut self, line_len: usize) {
        self.pos.down(self.adjust_col(line_len));
    }

    pub fn col(&self) -> usize {
        self.pos.col
    }

    pub fn row(&self) -> usize {
        self.pos.row
    }
}

type BufferLines<'a> = gap_buf::GapBufferLines<'a>;

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

    pub fn insert(&mut self, c: char) {
        self.buf.insert(&c.to_string());

        if c == '\n' {
            self.cursor.newline();
        } else {
            self.cursor.right();
        }
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

    pub fn lines_at(&'_ self, line_num: usize) -> BufferLines<'_> {
        self.buf.lines_at(line_num)
    }

    pub fn move_right(&mut self) {
        if let Some(ch) = self.buf.get_coord(self.cursor.row(), self.cursor.col() + 1)
            && ch != '\n'
        {
            self.cursor.right();
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor.col() == 0 {
            return;
        }

        self.cursor.left();
    }

    pub fn move_down(&mut self) {
        if let Some(line) = self.buf.line_at(self.cursor.row() + 1) {
            self.cursor.down(line.len());
        }
    }

    pub fn move_up(&mut self) {
        if let Some(line) = self.buf.line_at(self.cursor.row() - 1) {
            self.cursor.up(line.len());
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
            Mode::CommandLine => false,
        }
    }
}
