enum Mode {
    Normal,
    Insert,
    CommandLine,
}

pub struct Position {
    pub col: u16,
    pub row: u16,
}

pub struct Buffer {
    pub buf: String,
    pub cursor_position: Position,
}

impl Buffer {
    pub fn insert(self, c: char, pos: Position) -> Buffer {
        self
    }

    pub fn delete(self, pos: Position) -> Buffer {
        self
    }
}

pub struct Model {
    // TODO: I want to use larger ints for the cursor, but I need to handle wraparound
    // in the buffer first
    pub buffer: Buffer,
    pub mode: Mode,
}

impl Model {
    pub fn new(buffer: Buffer) -> Model {
        Model {
            mode: Mode::Normal,
            buffer: buffer,
        }
    }
}
