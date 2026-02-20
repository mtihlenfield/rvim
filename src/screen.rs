use crossterm::ExecutableCommand;
use crossterm::cursor;
use crossterm::terminal;
use std::io::stdout;

use crate::model::Model;

struct BufferView {}

impl BufferView {
    pub fn new() -> BufferView {
        BufferView {}
    }

    pub fn update(&mut self, model: &Model) -> std::io::Result<()> {
        stdout().execute(cursor::MoveTo(
            model.buffer.cursor_position.col,
            model.buffer.cursor_position.row,
        ))?;

        Ok(())
    }
}

pub struct Screen {
    buffer_view: BufferView,
    initialized: bool,
}

impl Screen {
    pub fn new() -> Screen {
        Screen {
            buffer_view: BufferView::new(),
            initialized: false,
        }
    }

    pub fn update(&mut self, model: &Model) -> std::io::Result<()> {
        if !self.initialized {
            terminal::enable_raw_mode()?;
            stdout().execute(terminal::Clear(terminal::ClearType::All))?;
            self.initialized = true;
        }

        self.buffer_view.update(model)
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode")
    }
}
