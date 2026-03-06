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
    offset: usize,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            // The position relative to the start of the buffer contents
            pos: Position::new(),
            // The global position as an index in to the buffer contents
            // TODO: Do I need this? Can I do pos.rows * pos.cols? The question is
            // how newlines are handled.
            offset: 0,
        }
    }

    pub fn newline(&mut self) {
        self.pos.newline();
        self.offset += 1;
    }

    pub fn right(&mut self) {
        self.pos.right();
        self.offset += 1;
    }

    pub fn left(&mut self) {
        self.pos.left();
        self.offset -= 1;
    }

    pub fn up(&mut self, col: usize) {
        self.pos.up(col);
        self.offset -= 1;
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
        // TODO: having to convert to string may not be a very good api
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
                // Note that we *know* at this point that we aren't at the very start of the
                // buffer. If we were we would have gotten a DeleteFromStart error.
                if self.cursor.col() == 0 {
                    // Subtracting 1 from the global cursor offset because we deleted
                    // a char but haven't updated the cursor yet, so it is out of date.
                    let cur_offset = self.cursor.offset - 1;

                    // Subtracting 1 here because the cursor is always one past the char
                    // that is being deleted
                    if cur_offset != 0
                        && let Some(prev_line_end) = self.buf.find_prev(cur_offset - 1, '\n')
                    {
                        let len = cur_offset - prev_line_end - 1;
                        self.cursor.up(len);
                    } else {
                        // We're on the first line, so we just set the cursor to the length
                        // of the buffer
                        self.cursor.up(cur_offset);
                    }
                } else {
                    self.cursor.left();
                }
                Ok(())
            }
            Err(gap_buf::GapBufferError::DeleteFromStart) => Ok(()),
            Err(e) => Err(BufferError(e.to_string())),
        }
    }

    pub fn lines_at(&'_ self, line_num: usize) -> BufferLines<'_> {
        self.buf.lines_at(line_num)
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
