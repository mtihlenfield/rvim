const DEFAULT_GAP_SIZE: usize = 64;

struct GapBuffer {
    buffer: Vec<char>,
    gap_start: usize,
    gap_end: usize,
}

impl GapBuffer {
    pub fn new() -> GapBuffer {
        GapBuffer {
            buffer: vec!['\0'; DEFAULT_GAP_SIZE],
            // index of the first writeable character in the gap
            gap_start: 0,
            // index of the first char after the gap - could be after the end of the vec
            gap_end: DEFAULT_GAP_SIZE,
        }
    }

    pub fn len(&self) -> usize {
        return self.buffer.len() - (self.gap_end - self.gap_start);
    }

    fn move_cursor(&mut self, pos: usize) {
        // how to handle it if the pos is beyond the end of the vec? Should probably fail right?
    }

    fn grow(&mut self) {
        // Grow the vector
        let new_gap = vec!['\0'; DEFAULT_GAP_SIZE];
        // Move the data at gap_start gap_size bytes to the right
        // Adjust gap start and gap end
    }

    /// Insert at the current cursor position
    pub fn insert(&mut self, s: &str) {
        for c in s.chars() {
            if self.gap_start >= self.gap_end {
                self.grow()
            }

            self.buffer[self.gap_start] = c;
            self.gap_start += 1;
        }
    }

    /// Insert at a new location in the buffer
    pub fn insert_at(&mut self, s: &str, pos: usize) {
        self.move_cursor(pos);
        self.insert(s);
    }

    /// Delete at the current cursor position
    pub fn delete(&mut self) {
        // TODO: what if gap_end = 0?
        self.gap_start -= 1;
    }
}

impl ToString for GapBuffer {
    // Required method
    fn to_string(&self) -> String {
        let mut buf = self.buffer.clone();
        buf.drain(self.gap_start..self.gap_end);
        buf.into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::GapBuffer;

    #[test]
    fn test_insert_string_in_empty() {
        let mut buf = GapBuffer::new();
        let hello = String::from("Hello, world");
        buf.insert(&hello);

        assert_eq!(buf.len(), hello.len());
        assert_eq!(buf.to_string(), hello);
    }

    #[test]
    fn test_basic_append() {
        let mut buf = GapBuffer::new();
        buf.insert("Hello,");
        buf.insert(" world");

        let hello = String::from("Hello, world");
        assert_eq!(buf.len(), hello.len());
        assert_eq!(buf.to_string(), hello);
    }

    #[test]
    fn test_basic_insert_at() {
        let mut buf = GapBuffer::new();
        buf.insert("Goodbye, world");
        buf.insert_at("creul, ", 7);

        let cruel = String::from("Hello, world");
        assert_eq!(buf.len(), cruel.len());
        assert_eq!(buf.to_string(), cruel);
    }

    #[test]
    fn test_basic_delete() {
        let mut buf = GapBuffer::new();
        let hello = String::from("Hello, world");
        buf.insert(&hello);
        buf.delete();

        assert_eq!(buf.to_string(), "Hello, worl");

        buf.delete();
        assert_eq!(buf.to_string(), "Hello, wor");
    }
}
