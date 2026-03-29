use crate::gap_buf;
use crossterm::event;
use log::info;

use crate::buffer::Buffer;

#[derive(Debug, Eq, PartialEq)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

pub struct Command {
    buff: gap_buf::GapBuffer,
    cursor_index: usize,
}

#[derive(Debug)]
pub struct CommandError(pub String);

impl Command {
    pub fn new() -> Command {
        Command {
            buff: gap_buf::GapBuffer::new(),
            cursor_index: 0,
        }
    }

    pub fn as_string(&self) -> String {
        self.buff.chars().collect()
    }

    pub fn insert(&mut self, c: char) {
        self.buff
            .insert_at(&c.to_string(), self.cursor_index)
            .expect("Failed to insert char");
        self.cursor_index += 1;
    }

    pub fn delete(&mut self) -> Result<(), CommandError> {
        match self.buff.delete_at(self.cursor_index) {
            Ok(()) => {
                self.cursor_index -= 1;
                Ok(())
            }
            Err(gap_buf::GapBufferError::DeleteFromStart) => Ok(()),
            Err(e) => Err(CommandError(e.to_string())),
        }
    }
}

pub struct EditorState {
    pub buffer: Buffer,
    pub command: Command,
    pub mode: Mode,
}

impl EditorState {
    pub fn new() -> EditorState {
        EditorState {
            mode: Mode::Normal,
            buffer: Buffer::new(),
            command: Command::new(),
        }
    }

    pub fn open_file(&mut self, path: &str) -> Result<(), std::io::Error> {
        self.buffer = Buffer::from_file(path)?;

        Ok(())
    }

    fn handle_normal_update(&mut self, key_ev: event::KeyEvent) -> bool {
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
                ':' => {
                    self.mode = Mode::Command;
                    info!("Switching to Command mode.");
                    false
                }
                _ => false,
            },
            _ => false,
        }
    }

    fn handle_insert_update(&mut self, key_ev: event::KeyEvent) -> bool {
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
                self.buffer.move_left();
                info!("Switching to Normal mode.");
            }
            _ => {}
        };

        false
    }

    fn handle_command_update(&mut self, key_ev: event::KeyEvent) -> bool {
        match key_ev.code {
            event::KeyCode::Char(c) => {
                if !c.is_control() {
                    self.command.insert(c);
                }
                false
            }
            event::KeyCode::Esc => {
                self.mode = Mode::Normal;
                info!("Switching to Normal mode.");
                false
            }
            event::KeyCode::Enter => {
                let should_exit = self.execute_command();
                self.mode = Mode::Normal;
                info!("Switching to Normal mode.");

                should_exit
            }
            event::KeyCode::Backspace => {
                if let Err(e) = self.command.delete() {
                    panic!("Failed to delete: {}", e.0);
                }
                false
            }
            _ => false,
        }
    }

    fn execute_command(&mut self) -> bool {
        false
    }

    pub fn update(&mut self, key_ev: event::KeyEvent) -> bool {
        match self.mode {
            Mode::Normal => self.handle_normal_update(key_ev),
            Mode::Insert => self.handle_insert_update(key_ev),
            Mode::Command => self.handle_command_update(key_ev),
        }
    }
}
