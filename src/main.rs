use crossterm::ExecutableCommand;
use crossterm::cursor;
use crossterm::event;
use crossterm::terminal;
use std::io::{Stdout, stdout};

enum Mode {
    Normal,
    Insert,
    CommandLine,
}

type Buffer = String;

struct Model {
    // TODO: I want to use larger ints for the cursor, but I need to handle wraparound
    // in the buffer first
    cursor_line: u16,
    cursor_col: u16,
    buffer: String,
    mode: Mode,
}

impl Model {
    pub fn new(buffer: Buffer) -> Model {
        Model {
            cursor_line: 0,
            cursor_col: 0,
            mode: Mode::Normal,
            buffer: buffer,
        }
    }
}

struct BufferView<'a> {
    // TODO: holds a buffer reference and screen position?
    out: &'a Stdout,
}

impl<'a> BufferView<'a> {
    pub fn new(out: &Stdout) -> BufferView {
        BufferView { out: out }
    }

    pub fn update(&mut self, model: &Model) {
        self.out
            .execute(cursor::MoveTo(model.cursor_line, model.cursor_col))
            .expect("Couldn't move cursor!");

        println!("{}", model.buffer);
    }
}

struct Window<'a> {
    buffer: BufferView<'a>,
    out: &'a Stdout,
}

impl<'a> Window<'a> {
    pub fn new(out: &Stdout) -> Window {
        Window {
            buffer: BufferView::new(out),
            out: out,
        }
    }

    pub fn update(&mut self, model: &Model) {
        self.buffer.update(model);
    }
}

struct Screen<'a> {
    window: Window<'a>,
    initialized: bool,
    out: &'a Stdout,
}

// TODO: should implement drop trait that takes us out of raw mode
impl<'a> Screen<'a> {
    pub fn new(out: &Stdout) -> Screen {
        Screen {
            window: Window::new(out),
            initialized: false,
            out: out,
        }
    }

    pub fn update(&mut self, model: &Model) {
        if !self.initialized {
            terminal::enable_raw_mode().expect("Could not turn on Raw mode");
            self.out
                .execute(terminal::Clear(terminal::ClearType::All))
                .expect("Could not clear");
            self.initialized = true;
        }

        self.window.update(model);
    }
}

impl<'a> Drop for Screen<'a> {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Unable to disable raw mode")
    }
}

fn main() {
    let out = stdout();
    let mut screen = Screen::new(&out);
    let mut model = Model::new("hello, world".to_string());
    screen.update(&model);

    loop {
        if let event::Event::Key(ev) = event::read().expect("Failed to read line") {
            match ev {
                event::KeyEvent {
                    code: event::KeyCode::Char('q'),
                    modifiers: event::KeyModifiers::NONE,
                    ..
                } => break,
                _ => {
                    screen.update(&model);
                }
            }
            println!("{:?}\r", ev);
        };
    }
}
