use crossterm::{ExecutableCommand, QueueableCommand, cursor, style, terminal};
use log::{info, warn};
use std::io::{Stdout, Write, stdout};

use crate::position::Position;
use crate::state;

struct BufferView {
    anchor: usize,
    cursor: Position,
}

impl BufferView {
    pub fn new() -> BufferView {
        BufferView {
            anchor: 0,
            cursor: Position::new(),
        }
    }

    fn set_cursor(&mut self, row: u16, col: u16) {
        self.cursor.row = row as usize;
        self.cursor.col = col as usize;
    }

    fn update_anchor(&mut self, screen_buf: &ScreenBuf, buffer: &state::Buffer) {
        // TODO: what about a case where the file is one long line that is larger than the
        // screen buff? To handle this, we probably need ot actually use the cols part of the
        // anchor to track which part of the line we are at. Should basically always be 0 except in
        // this one case.
        // let global_cursor_row = buffer.cursor.row();
        // let cols = screen_buf.cols;
        // let rows = screen_buf.rows;
        // TODO: make sure to account for the fact that we preserve the last row

        // // Check if cursor is above the anchor
        // if global_cursor_row <= self.anchor.row {
        //     self.anchor.row = global_cursor_row;
        //     return;
        // }

        // // Count the number of screen lines between the anchor and the cursor.
        // let screen_lines_to_cursor: usize = buffer
        //     .lines_at(self.anchor.row)
        //     .expect(&format!("Buffer has no line {}", self.anchor.row))
        //     .take(global_cursor_row - self.anchor.row + 1)
        //     .map(|line| line.len().div_ceil(cols as usize))
        //     .sum();

        // if screen_lines_to_cursor <= rows as usize {
        //     // Cursor is within the visible window, nothing to do
        //     return;
        // }

        // let mut screen_line_count = 0;
        // let mut line_count = 0;
        // let mut move_anchor = false;
        // let iter = buffer
        //     .lines_at_rev(global_cursor_row)
        //     .expect("Buffer has no line {global_cursor_row}");
        // for line in iter {
        //     screen_line_count += line.len().div_ceil(screen_buf.cols as usize);
        //     if screen_line_count > screen_buf.rows as usize {
        //         move_anchor = true;
        //         break;
        //     }

        //     line_count += 1;
        // }

        // if !move_anchor {
        //     return;
        // }

        // self.anchor.row = global_cursor_row - line_count;
    }

    pub fn update(
        &mut self,
        screen_buf: &mut ScreenBuf,
        buffer_state: &state::Buffer,
    ) -> std::io::Result<()> {
        let max_col = screen_buf.cols - 1;
        // preserve the last row for the status line
        let max_row = screen_buf.rows - 2;
        self.update_anchor(screen_buf, &buffer_state);

        if buffer_state.is_empty() {
            self.fill_empty_lines(screen_buf, 0);
            return Ok(());
        }

        // Note that were assuming that because we've updated the anchor already, the buffer cursor must
        // be >= the anchor
        let cursor_offset = buffer_state.cursor.index - self.anchor;

        let mut row: u16 = 0;
        let mut col: u16 = 0;
        for (offset, ch) in buffer_state.chars_at(self.anchor).enumerate() {
            // preserve the last row for the status line
            if row > max_row {
                break;
            }

            if ch == '\n' {
                col = 0;
                if offset == cursor_offset {
                    self.set_cursor(row, col);
                }
                row += 1;
                continue;
            }

            if col >= max_col {
                if row == max_row {
                    // We can't display the whole line, so we won't display an of it.
                    // vim puts 3 `@` symbols on the line. Might be good to do that
                    screen_buf.clear_row(row);
                    break;
                }

                row += 1;
                col = 0;
            }

            if offset == cursor_offset {
                self.set_cursor(row, col);
            }

            screen_buf.write(row, col, ch);
            col += 1;
        }

        self.fill_empty_lines(screen_buf, row);

        Ok(())
    }

    pub fn fill_empty_lines(&self, screen_buf: &mut ScreenBuf, start_row: u16) {
        for empty_row in start_row..screen_buf.rows - 1 {
            screen_buf.write(empty_row, 0, '~');
        }
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

    pub fn clear_row(&mut self, row: u16) {
        for c in &mut self.back[row as usize] {
            *c = ' '
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
        self.buffer_view
            .update(&mut self.screen_buf, &new_state.buffer)?;
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
