use crossterm::ExecutableCommand;
use crossterm::cursor;
use crossterm::terminal;
use log::{info, warn};
use std::io::{Write, stdout};

use crate::model;

struct BufferView {}

impl BufferView {
    pub fn new() -> BufferView {
        BufferView {}
    }

    pub fn update(&mut self, new_model: &model::Model) -> std::io::Result<()> {
        let mut out = stdout();
        out.execute(cursor::MoveTo(0, 0))?;
        write!(out, "{}", &new_model.buffer.buf.to_string())?;

        info!("Writing: {}", &new_model.buffer.buf.to_string());
        info!("position: {:?}", new_model.buffer.cursor_position);

        out.execute(cursor::MoveTo(
            new_model.buffer.cursor_position.col,
            new_model.buffer.cursor_position.row,
        ))?;

        out.flush()?;

        Ok(())
    }
}

struct StatusView {}

impl StatusView {
    pub fn new() -> StatusView {
        StatusView {}
    }

    pub fn update(&mut self, new_model: &model::Model) -> std::io::Result<()> {
        // let mut out = stdout();

        // match new_model.mode {
        //     model::Mode::Insert => write!(out, "{}", "-- INSERT --")?,
        //     _ => {}
        // }

        // out.flush()?;

        Ok(())
    }
}

pub struct Screen {
    buffer_view: BufferView,
    status_view: StatusView,
    initialized: bool,
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            buffer_view: BufferView::new(),
            status_view: StatusView::new(),
            initialized: false,
        }
    }

    pub fn update(&mut self, new_model: &model::Model) -> std::io::Result<()> {
        if !self.initialized {
            terminal::enable_raw_mode()?;
            self.initialized = true;
        }

        stdout().execute(terminal::Clear(terminal::ClearType::All))?;
        self.buffer_view.update(new_model)?;
        self.status_view.update(new_model)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        if let Err(_) = terminal::disable_raw_mode() {
            warn!("Failed to disable raw mode on close.")
        }

        if let Err(_) = stdout().execute(terminal::Clear(terminal::ClearType::All)) {
            warn!("Failed to clear screen on close.")
        }
    }
}
