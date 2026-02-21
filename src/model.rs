use crate::gap_buf;
use crossterm::event;
use log::info;

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

type BufferIter<'a> = gap_buf::GapBufferIter<'a>;

#[derive(Debug)]
pub struct BufferError(String);

pub struct Buffer {
    buf: gap_buf::GapBuffer,
    cursor_position: Position,
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

    pub fn delete(&mut self) -> Result<(), BufferError> {
        match self.buf.delete() {
            Ok(()) => {
                self.cursor_position.col -= 1;
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
        }
    }
}
