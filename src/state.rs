use crate::gap_buf;
use crossterm::event;
use log::info;

pub enum Mode {
    Normal,
    Insert,
    CommandLine,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub row: u16,
    pub col: u16,
}

impl Position {
    pub fn new() -> Position {
        Position { row: 0, col: 0 }
    }

    pub fn newline(&mut self) {
        self.row += 1;
        self.col = 0;
    }

    pub fn right(&mut self) {
        self.col += 1;
    }

    pub fn left(&mut self) {
        self.col -= 1;
    }
}

type BufferIter<'a> = gap_buf::GapBufferIter<'a>;

#[derive(Debug)]
pub struct BufferError(String);

pub struct Buffer {
    buf: gap_buf::GapBuffer,
    pub cursor: Position,
    view_rows: u16,
    view_cols: u16,
}

impl Buffer {
    pub fn new(view_rows: u16, view_cols: u16) -> Buffer {
        Buffer {
            buf: gap_buf::GapBuffer::new(),
            cursor: Position::new(),
            view_rows: view_rows,
            view_cols: view_cols,
        }
    }

    pub fn insert(&mut self, c: char) {
        // TODO: having to convert to string may not be a very good api
        self.buf.insert(&c.to_string());

        if c == '\n' || self.cursor.col >= self.view_cols {
            self.cursor.newline();
        } else {
            self.cursor.right();
        }

        info!("Cursor afte insert: {:?}", self.cursor);
    }

    pub fn delete(&mut self) -> Result<(), BufferError> {
        match self.buf.delete() {
            Ok(()) => {
                // TODO: this does not currently handle moving the cursor when you are deleting the
                // last remaining char in a row. To do that, I may need to know where on the screen
                // the previous newline was.
                self.cursor.left();
                Ok(())
            }
            Err(gap_buf::GapBufferError::DeleteFromStart) => Ok(()),
            Err(e) => Err(BufferError(e.to_string())),
        }
    }

    pub fn iter(&'_ self) -> BufferIter<'_> {
        self.buf.iter()
    }
}

pub struct EditorState {
    pub buffer: Buffer,
    pub mode: Mode,
}

impl EditorState {
    pub fn new(view_rows: u16, view_cols: u16) -> EditorState {
        EditorState {
            mode: Mode::Normal,
            buffer: Buffer::new(view_rows, view_cols),
        }
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
