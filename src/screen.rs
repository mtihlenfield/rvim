use crossterm::{ExecutableCommand, QueueableCommand, cursor, style, terminal};
use log::{info, warn};
use std::io::{Stdout, Write, stdout};

use crate::position::Position;
use crate::state;

struct BufferView {
    anchor: Position,
    cursor: Position,
}

impl BufferView {
    pub fn new() -> BufferView {
        BufferView {
            anchor: Position::new(),
            cursor: Position::new(),
        }
    }

    fn update_anchor(&mut self, screen_buf: &ScreenBuf, global_cursor: &state::Cursor) {
        // Given the current anchor position and the screen buff size, we should be able to
        // determine if the current cursor can fit is in the view.
        //
        // If we determine that we can't fit it in the view, we probably have to scan from the
        // anchor to the cursor to see how many screen lines we need. As we walk we keep track of
        // the screen line and the global line.
        //
        // Then we walk back from the cursor. We want to anchor on the first which we can
        // completely display.
    }

    pub fn update(
        &mut self,
        screen_buf: &mut ScreenBuf,
        new_state: &state::EditorState,
    ) -> std::io::Result<()> {
        let max_col = screen_buf.cols - 1;
        let max_row = screen_buf.rows - 1;
        self.update_anchor(screen_buf, &new_state.buffer.cursor);

        // For now we will set the cursor match the global cursor, but we may have to update
        // it if the line is long enough to wrap around
        self.cursor.col = new_state.buffer.cursor.col();
        // We've moved the anchor at this point, so it's safe to assume that the cursor
        // row is greater than the anchor row
        self.cursor.row = new_state.buffer.cursor.row() - self.anchor.row;

        let mut row: u16 = 0;
        let mut col: u16 = 0;
        // TODO: right now this is an log(n) search through the buffer - way too inefficient
        for line in new_state.buffer.lines_at(self.anchor.row) {
            // preserve the last row for the status line
            if row > max_row - 1 {
                break;
            }

            for ch in line.chars() {
                if col > max_col {
                    row += 1;
                    col = 0;
                }

                screen_buf.write(row, col, ch);
                col += 1;
            }

            row += 1;
            col = 0;

            if row > max_row {
                break;
            }
        }

        // TODO: update the cursor. Where we place it depends on partially on:
        // a) the mode
        // b) what line wraparounds happened. We probably need to track wraparounds that
        // occurred on a line <= the cursor row

        for empty_row in row..screen_buf.rows - 1 {
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
                for (i, c) in "-- INSERT --".chars().enumerate() {
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
        let cursor = match new_state.mode {
            state::Mode::Insert | state::Mode::Normal => self.buffer_view.cursor.clone(),
            _ => Position::new(),
        };

        out.queue(cursor::MoveTo(cursor.col as u16, cursor.row as u16))?;

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
