use crossterm::{ExecutableCommand, QueueableCommand, cursor, style, terminal};
use log::{info, warn};
use std::io::{Stdout, Write, stdout};

use crate::buffer;
use crate::position::Position;
use crate::state;

fn screen_line_count(line: &buffer::BufferSlice, max_col: u16) -> usize {
    let str_line: String = line.chars().collect();
    let stripped = str_line.replace('\n', "");
    stripped.len().div_ceil(max_col.into()).max(1)
}

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

    fn scroll_in_to_view(&mut self, buffer: &buffer::Buffer, max_row: u16, max_col: u16) {
        // TODO: this is not handling the case where the file is one big line that fills
        // more than one screen.
        if buffer.len() == 0 {
            self.anchor = 0;
            return;
        }

        let global_cursor = buffer.cursor_index();
        // If the cursor is above the anchor, search for the start of the cursor line and
        // put the anchor there.
        if global_cursor <= self.anchor {
            self.anchor = buffer.line_start(global_cursor);
            return;
        }

        let max_chars = global_cursor - buffer.line_start(self.anchor);

        let mut row = 0;
        let mut col = 0;
        for ch in buffer.chars_at(self.anchor).take(max_chars) {
            if ch == '\n' {
                col = 0;
                row += 1;
                continue;
            }

            if col >= max_col {
                row += 1;
                col = 0;
            }

            col += 1;
        }

        if row <= max_row {
            return;
        }

        let cursor_line_end = if global_cursor == buffer.len() {
            // When in insert mode, the cursor can be one char past the buffer
            global_cursor - 1
        } else {
            buffer.line_end(global_cursor)
        };

        let mut offset = 0;
        let mut total_screen_lines = 0;
        // if it is not in the current window, we walk backwards from the *end* of the line that the
        // cursor is on until the screen buff is filled
        for line in buffer.lines_at_char_rev(cursor_line_end) {
            let screen_lines = screen_line_count(&line, max_col);

            if total_screen_lines + screen_lines > (max_row + 1).into() {
                break;
            }

            offset = line.start();
            total_screen_lines += screen_lines;
        }

        self.anchor = offset;
    }

    pub fn update(
        &mut self,
        screen_buf: &mut ScreenBuf,
        buffer_state: &buffer::Buffer,
    ) -> std::io::Result<()> {
        let max_col = screen_buf.cols - 1;
        // preserve the last row for the status line
        let max_row = screen_buf.rows - 2;
        self.scroll_in_to_view(&buffer_state, max_row, max_col);

        if buffer_state.is_empty() {
            self.set_cursor(0, 0);
            self.fill_empty_lines(screen_buf, 1);
            return Ok(());
        }

        // Note that were assuming that because we've updated the anchor already, the buffer cursor must
        // be >= the anchor
        let cursor_offset = buffer_state.cursor_index() - self.anchor;
        let mut cursor_set = false;

        let mut row: u16 = 0;
        let mut col: u16 = 0;
        for (offset, ch) in buffer_state.chars_at(self.anchor).enumerate() {
            if row > max_row {
                break;
            }

            if ch == '\n' {
                if offset == cursor_offset {
                    cursor_set = true;
                    // if we enter insert mode by hitting `a`, this could put the cursor at a \n.
                    // In that case we want the cursor one past the end of the line instead of at
                    // 0. If the previous char was not a \n, then we know we are in append mode
                    let cursor_col =
                        if buffer_state.get(cursor_offset.saturating_sub(1)) != Some('\n') {
                            col + 1
                        } else {
                            0
                        };
                    self.set_cursor(row, cursor_col);
                }

                col = 0;
                row += 1;
                continue;
            }

            // TODO: this *should* be col > max_col. Right now with >= we wrap around one char too
            // early. However using > exposes a cursor bug which causes it to not wrap around
            // correctly. So I'm leaving >= until I fix the cursor bug.
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
                cursor_set = true;
                self.set_cursor(row, col);
            }

            screen_buf.write(row, col, ch);
            col += 1;
        }

        if !cursor_set {
            // The cursor is at the end of the buffer - happens in insert mode.
            self.set_cursor(row, col);
        }

        self.fill_empty_lines(screen_buf, row + 1);

        Ok(())
    }

    pub fn fill_empty_lines(&self, screen_buf: &mut ScreenBuf, start_row: u16) {
        for empty_row in start_row..screen_buf.rows - 1 {
            screen_buf.write(empty_row, 0, '~');
        }
    }
}

struct StatusView {
    cursor: Position,
}

impl StatusView {
    pub fn new() -> StatusView {
        StatusView {
            cursor: Position::new(),
        }
    }

    pub fn update(
        &mut self,
        screen_buf: &mut ScreenBuf,
        new_state: &state::EditorState,
    ) -> std::io::Result<()> {
        let mode_str = match new_state.mode {
            state::Mode::Insert => "-- INSERT --".to_string(),
            state::Mode::Normal => {
                if new_state.has_error() {
                    new_state.command.as_string()
                } else {
                    "-- NORMAL --".to_string()
                }
            }
            state::Mode::Command => format!(":{}", new_state.command.as_string()),
        };

        self.cursor = if new_state.mode == state::Mode::Command {
            Position {
                row: (screen_buf.rows - 1).into(),
                // NOTE: not bothering with command wraparound right now. Don't have commands
                // longer than 2 chars right now
                col: mode_str.len(),
            }
        } else {
            Position::new()
        };

        for (i, c) in mode_str.chars().enumerate() {
            screen_buf.write(screen_buf.rows - 1, i as u16, c);
        }

        Ok(())
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

    pub fn resize(&mut self, rows: u16, cols: u16) -> std::io::Result<()> {
        self.screen_buf = ScreenBuf::new(rows, cols);
        self.buffer_view = BufferView::new();
        self.status_view = StatusView::new();

        let mut out = stdout();
        out.execute(terminal::Clear(terminal::ClearType::All))?;
        Ok(())
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
            state::Mode::Command => self.status_view.cursor.clone(),
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
