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
    pub row: usize,
    pub col: usize,
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

    pub fn up(&mut self, col: usize) {
        self.col = col;
        self.row -= 1;
    }
}

#[derive(Debug, Clone)]
pub struct Cursor {
    viewport_pos: Position,
    global_pos: Position,
    global_offset: usize,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            // The position relative to the start of the buffer view
            viewport_pos: Position::new(),
            // The position relative to the start of the buffer contents
            global_pos: Position::new(),
            // The global position as an index in to the buffer contents
            // TODO: Do I need this? Can I do global_pos.rows * global_pos.cols? The question is
            // how newlines are handled.
            global_offset: 0,
        }
    }

    pub fn newline(&mut self) {
        self.viewport_pos.newline();
        self.global_pos.newline();
        self.global_offset += 1;
    }

    pub fn right(&mut self) {
        self.viewport_pos.right();
        self.global_pos.right();
        self.global_offset += 1;
    }

    pub fn left(&mut self) {
        self.viewport_pos.left();
        self.global_pos.left();
        self.global_offset -= 1;
    }

    pub fn up(&mut self, col: usize) {
        self.viewport_pos.up(col);
        self.global_pos.up(col);
        self.global_offset -= 1;
    }

    /// Return the current global column. Note that this should be the same
    /// as the window column. This just returns a usize instead of u16.
    pub fn col(&self) -> usize {
        self.global_pos.col
    }

    pub fn viewport_col(&self) -> u16 {
        self.viewport_pos.col as u16
    }

    pub fn viewport_row(&self) -> u16 {
        self.viewport_pos.row as u16
    }
}

type BufferIter<'a> = gap_buf::GapBufferIter<'a>;

#[derive(Debug)]
pub struct BufferError(String);

pub struct Buffer {
    buf: gap_buf::GapBuffer,
    pub cursor: Cursor,
    view_rows: u16,
    view_cols: u16,
}

impl Buffer {
    pub fn new(view_rows: u16, view_cols: u16) -> Buffer {
        Buffer {
            buf: gap_buf::GapBuffer::new(),
            cursor: Cursor::new(),
            view_rows: view_rows,
            view_cols: view_cols,
        }
    }

    pub fn insert(&mut self, c: char) {
        // TODO: having to convert to string may not be a very good api
        self.buf.insert(&c.to_string());

        if c == '\n' || self.cursor.col() >= self.view_cols.into() {
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
                    let cur_offset = self.cursor.global_offset - 1;

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

    pub fn iter(&'_ self) -> BufferIter<'_> {
        self.buf.chars()
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
