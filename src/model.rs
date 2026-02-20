use crate::gap_buf;
use crossterm::event;

pub enum Mode {
    Normal,
    Insert,
    // CommandLine,
}

#[derive(Debug)]
pub struct Position {
    pub col: u16,
    pub row: u16,
}

impl Position {
    pub fn new() -> Position {
        Position { col: 0, row: 0 }
    }
}

pub struct Buffer {
    buf: gap_buf::GapBuffer,
    pub cursor_position: Position,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            buf: gap_buf::GapBuffer::new(),
            cursor_position: Position::new(),
        }
    }

    pub fn insert(&mut self, c: char) {
        self.buf.insert(&c.to_string());
        self.cursor_position.col += 1;
    }

    pub fn delete(&mut self) {
        if let Ok(()) = self.buf.delete() {
            // We should only get an Err back if we can't delete
            // because we're at the start of the buffer
            self.cursor_position.col -= 1;
        }
    }
}

pub struct Model {
    pub buffer: Buffer,
    pub mode: Mode,
}

impl Model {
    pub fn new() -> Model {
        Model {
            mode: Mode::Normal,
            buffer: Buffer::new(),
        }
    }

    pub fn handle_normal_update(&mut self, key_ev: event::KeyEvent) -> bool {
        match key_ev.code {
            event::KeyCode::Char(c) => match c {
                'q' => true,
                'i' => {
                    self.mode = Mode::Insert;
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
                // TODO: should probably ignore control chars
                self.buffer.insert(c);
            }
            event::KeyCode::Backspace => {
                self.buffer.delete();
            }
            event::KeyCode::Esc => {
                self.mode = Mode::Normal;
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
