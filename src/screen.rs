use crossterm::{ExecutableCommand, QueueableCommand, cursor, style, terminal};
use log::warn;
use std::io::{Stdout, Write, stdout};

use crate::state;

struct BufferView {}

impl BufferView {
    pub fn new() -> BufferView {
        BufferView {}
    }

    pub fn update(
        &mut self,
        screen_buf: &mut ScreenBuf,
        new_state: &state::EditorState,
    ) -> std::io::Result<()> {
        let mut row = 0;
        let mut col = 0;
        for c in new_state.buffer.iter() {
            if *c == '\n' || col >= screen_buf.cols {
                row += 1;
                col = 0;
                continue;
            }

            screen_buf.write(row, col, *c);
            col += 1;
        }

        for empty_row in row + 1..screen_buf.rows - 1 {
            screen_buf.write(empty_row, 0, '~');
        }

        Ok(())
    }
}

struct StatusView {}

impl StatusView {
    pub fn new() -> StatusView {
        StatusView {}
    }

    pub fn update(
        &mut self,
        screen_buf: &mut ScreenBuf,
        new_state: &state::EditorState,
    ) -> std::io::Result<()> {
        match new_state.mode {
            state::Mode::Insert => {
                for (i, c) in "-- Insert --".chars().enumerate() {
                    screen_buf.write(screen_buf.rows - 1, i as u16, c);
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }
}

struct ScreenBuf {
    rows: u16,
    cols: u16,
    back: Vec<Vec<char>>,
    front: Vec<Vec<char>>,
}

impl ScreenBuf {
    pub fn new(rows: u16, cols: u16) -> ScreenBuf {
        ScreenBuf {
            rows: rows,
            cols: cols,
            back: vec![vec![' '; cols as usize]; rows as usize],
            front: vec![vec![' '; cols as usize]; rows as usize],
        }
    }

    pub fn clear(&mut self) {
        for row in &mut self.back {
            for c in row {
                *c = ' ';
            }
        }
    }

    pub fn write(&mut self, row: u16, col: u16, val: char) {
        self.back[row as usize][col as usize] = val;
    }

    pub fn flush(&mut self, out: &mut Stdout) -> std::io::Result<()> {
        // TODO: make sure this traversal is cache friendly
        for row in 0..self.rows {
            for col in 0..self.cols {
                let front_col = self.front[row as usize][col as usize];
                let back_col = self.back[row as usize][col as usize];
                if back_col != front_col {
                    out.queue(cursor::MoveTo(col, row))?;
                    out.queue(style::Print(back_col))?;
                    self.front[row as usize][col as usize] = back_col.clone();
                }
            }
        }
        Ok(())
    }
}

pub struct Screen {
    screen_buf: ScreenBuf,
    buffer_view: BufferView,
    status_view: StatusView,
    initialized: bool,
}

impl Screen {
    pub fn new(rows: u16, cols: u16) -> Screen {
        Screen {
            screen_buf: ScreenBuf::new(rows, cols),
            buffer_view: BufferView::new(),
            status_view: StatusView::new(),
            initialized: false,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.screen_buf = ScreenBuf::new(width, height);
    }

    pub fn update(&mut self, new_state: &state::EditorState) -> std::io::Result<()> {
        let mut out = stdout();
        if !self.initialized {
            terminal::enable_raw_mode()?;
            out.execute(terminal::EnterAlternateScreen)?;
            out.execute(terminal::Clear(terminal::ClearType::All))?;
            self.initialized = true;
        }

        self.screen_buf.clear();
        self.buffer_view.update(&mut self.screen_buf, new_state)?;
        self.status_view.update(&mut self.screen_buf, new_state)?;
        self.screen_buf.flush(&mut out)?;
        match new_state.mode {
            state::Mode::Insert | state::Mode::Normal => {
                let cursor = new_state.buffer.cursor.clone();
                out.queue(cursor::MoveTo(cursor.viewport_col(), cursor.viewport_row()))?;
            }
            _ => {}
        };
        out.flush()
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        if let Err(_) = stdout().execute(terminal::Clear(terminal::ClearType::All)) {
            warn!("Failed to clear screen on close.");
        }

        if let Err(_) = stdout().execute(terminal::LeaveAlternateScreen) {
            warn!("Failed to return to alt screen.");
        }

        if let Err(_) = terminal::disable_raw_mode() {
            warn!("Failed to disable raw mode on close.");
        }
    }
}
