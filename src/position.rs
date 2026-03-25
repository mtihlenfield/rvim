#[derive(Debug, Clone)]
pub struct Position {
    pub row: usize,
    pub col: usize,
}

impl Position {
    pub fn new() -> Position {
        Position { row: 0, col: 0 }
    }

    // pub fn newline(&mut self) {
    //     self.row += 1;
    //     self.col = 0;
    // }

    // pub fn right(&mut self) {
    //     self.col += 1;
    // }

    // pub fn left(&mut self) {
    //     self.col -= 1;
    // }

    // pub fn up(&mut self, col: usize) {
    //     self.col = col;
    //     self.row -= 1;
    // }

    // pub fn down(&mut self, col: usize) {
    //     self.col = col;
    //     self.row += 1;
    // }
}
