use crossterm::terminal;
use crossterm::{ExecutableCommand, QueueableCommand};
use log::warn;
use std::io::{Stdout, Write, stdout};

use crate::model;

struct BufferView {}

impl BufferView {
    pub fn new() -> BufferView {
        BufferView {}
    }

    pub fn update(
        &mut self,
        screen_buf: &mut ScreenBuf,
        new_model: &model::Model,
    ) -> std::io::Result<()> {
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
        new_model: &model::Model,
    ) -> std::io::Result<()> {
        // TODO: add the buffer contents
        Ok(())
    }
}

struct ScreenBuf {
    width: u16,
    heigh: u16,
    back: Vec<Vec<char>>,
    front: Vec<Vec<char>>,
}

impl ScreenBuf {
    pub fn new(width: u16, height: u16) -> ScreenBuf {
        ScreenBuf {
            width: width,
            heigh: height,
            back: vec![vec![' '; width as usize]; height as usize],
            front: vec![vec![' '; width as usize]; height as usize],
        }
    }

    pub fn clear(&mut self) {
        // TODO:
        // for row in &mut self.back {
        //     for c in &mut row {
        //         *c = ' ';
        //     }
        // }
    }

    pub fn write(&mut self, col: u16, row: u16, val: char) {}

    pub fn flush(&self, out: &Stdout) -> std::io::Result<()> {
        // TODO: look at some other editors and see how they handle this
        // for row in 0..self.width {
        //     // TODO: make sure this traversal is cache friendly
        //     for row in 0..self.height {
        //         for col in 0..self.width {
        //             let front_col = self.front[row][col];
        //             let back_col = self.back[row][col];
        //         }
        //     }
        // }
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
    pub fn new(width: u16, height: u16) -> Screen {
        Screen {
            screen_buf: ScreenBuf::new(width, height),
            buffer_view: BufferView::new(),
            status_view: StatusView::new(),
            initialized: false,
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.screen_buf = ScreenBuf::new(width, height);
    }

    pub fn update(&mut self, new_model: &model::Model) -> std::io::Result<()> {
        let mut out = stdout();
        if !self.initialized {
            // TODO enter alt screen so we don't mess up the term history

            out.execute(terminal::EnterAlternateScreen)?;
            out.execute(terminal::Clear(terminal::ClearType::All))?;
            terminal::enable_raw_mode()?;
            self.initialized = true;
        }

        self.screen_buf.clear();
        self.buffer_view.update(&mut self.screen_buf, new_model)?;
        self.status_view.update(&mut self.screen_buf, new_model)?;
        self.screen_buf.flush(&out)?;
        out.flush()
    }
}

impl Drop for Screen {
    fn drop(&mut self) {
        if let Err(_) = terminal::disable_raw_mode() {
            warn!("Failed to disable raw mode on close.");
        }

        if let Err(_) = stdout().execute(terminal::Clear(terminal::ClearType::All)) {
            warn!("Failed to clear screen on close.");
        }

        if let Err(_) = stdout().execute(terminal::LeaveAlternateScreen) {
            warn!("Failed to return to alt screen.");
        }
    }
}
