use crossterm::event;
use log::info;

use crate::buffer::Buffer;

pub enum Mode {
    Normal,
    Insert,
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
                'a' => {
                    self.mode = Mode::Insert;
                    self.buffer.move_right(true);
                    info!("Switching to Insert mode.");
                    false
                }
                'l' => {
                    self.buffer.move_right(false);
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
