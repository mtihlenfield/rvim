const DEFAULT_GAP_SIZE: usize = 64;

// TODO: make this enum with different error types, impl Error and Display
#[derive(Debug)]
pub struct MoveError(&'static str);

#[derive(Debug)]
pub struct GapBuffer {
    // TODO: using char means I'm using 4x memory than a u8...
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
            // index of the char after the gap
            gap_end: DEFAULT_GAP_SIZE,
        }
    }

    pub fn len(&self) -> usize {
        self.buffer.len() - (self.gap_end - self.gap_start)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn gap_size(&self) -> usize {
        self.gap_end - self.gap_start
    }

    fn move_cursor(&mut self, pos: usize) -> Result<(), MoveError> {
        if pos > self.len() {
            return Err(MoveError(
                "Cannot move the gap buffer cursor past the data end.",
            ));
        }

        let distance = pos.abs_diff(self.gap_start);
        match self.gap_start.cmp(&pos) {
            std::cmp::Ordering::Less => {
                self.buffer
                    .copy_within(self.gap_end..self.gap_end + distance, self.gap_start);
                self.gap_start += distance;
                self.gap_end += distance;
            }
            std::cmp::Ordering::Greater => {
                self.buffer.copy_within(
                    self.gap_start - distance..self.gap_start,
                    self.gap_end - distance,
                );
                self.gap_start -= distance;
                self.gap_end -= distance;
            }
            std::cmp::Ordering::Equal => return Ok(()),
        };

        Ok(())
    }

    pub fn grow_gap(&mut self, required_size: usize) {
        let alloc_size = required_size + DEFAULT_GAP_SIZE;
        let old_size = self.buffer.len();

        // This puts appends to the end
        self.buffer.resize(old_size + alloc_size, '\0');
        self.buffer
            .copy_within(self.gap_end..old_size, self.gap_end + alloc_size);
        self.gap_end += alloc_size;
    }

    /// Insert at the current cursor position
    pub fn insert(&mut self, s: &str) {
        let current_gap_size = self.gap_size();
        let str_len = s.chars().count();
        if current_gap_size < str_len {
            self.grow_gap(str_len - current_gap_size);
        }

        for c in s.chars() {
            self.buffer[self.gap_start] = c;
            self.gap_start += 1;
        }
    }

    /// Insert at a new location in the buffer
    pub fn insert_at(&mut self, s: &str, pos: usize) -> Result<(), MoveError> {
        self.move_cursor(pos)?;
        self.insert(s);

        Ok(())
    }

    /// Delete at the current cursor position
    pub fn delete(&mut self) -> Result<(), MoveError> {
        if self.gap_start == 0 {
            return Err(MoveError("Cannot delete from beginning of buffer."));
        }

        self.gap_start -= 1;

        Ok(())
    }

    /// Translates an index in to the GapBuffer in to index into
    /// the "real" inner buffer. Does not do bounds checking on the input
    /// index, so it may return a "real" index that is outside the bounds
    /// of the GapBuffer if the passed in index is not valid.
    fn translate_index(&self, index: usize) -> usize {
        if index < self.gap_start {
            index
        } else {
            index + self.gap_size()
        }
    }

    pub fn get(&self, index: usize) -> Option<&char> {
        let real_index = self.translate_index(index);
        self.buffer.get(real_index)
    }

    pub fn iter(&'_ self) -> GapBufferIter<'_> {
        GapBufferIter {
            buff: self,
            index: 0,
        }
    }
}

impl Default for GapBuffer {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for GapBuffer {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        for &c in &self.buffer[..self.gap_start] {
            write!(f, "{}", c)?;
        }

        for &c in &self.buffer[self.gap_end..] {
            write!(f, "{}", c)?;
        }

        Ok(())
    }
}

pub struct GapBufferIter<'a> {
    buff: &'a GapBuffer,
    index: usize,
}

impl<'a> Iterator for GapBufferIter<'a> {
    type Item = &'a char;

    #[inline]
    fn next(&mut self) -> Option<&'a char> {
        let next = self.buff.get(self.index);
        if next.is_some() {
            self.index += 1;
        }
        next
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_string_in_empty() {
        let mut buf = GapBuffer::new();
        let hello = String::from("Hello, world");
        buf.insert(&hello);

        assert_eq!(buf.len(), hello.len());
        assert_eq!(buf.to_string(), hello);
    }

    #[test]
    fn insert_fill_from_empty() {
        let mut buf = GapBuffer::new();
        let hello = "a".repeat(DEFAULT_GAP_SIZE);
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
        buf.insert("Hello, world");

        // Testing moving the cursor from the end to the middle of a string
        buf.insert_at("cruel ", 7).expect("Should work");

        let cruel = String::from("Hello, cruel world");
        assert_eq!(buf.to_string(), cruel);

        // Testing moving from the middle to the end of a string
        buf.insert_at(".", 18).expect("Should work");
        let cruel = String::from("Hello, cruel world.");
        assert_eq!(buf.to_string(), cruel);

        // Moving from the end to the beginning
        buf.insert_at("Goodbye ", 0).expect("Should work");
        let cruel = String::from("Goodbye Hello, cruel world.");
        assert_eq!(buf.to_string(), cruel);

        // inserting at 0 when cursor == 0;
        buf.insert_at("Goodbye ", 0).expect("Should work");
        let cruel = String::from("Goodbye Goodbye Hello, cruel world.");
        assert_eq!(buf.to_string(), cruel);
    }

    #[test]
    fn test_insert_with_grow() {
        let mut buf = GapBuffer::new();
        let start = "a".repeat(DEFAULT_GAP_SIZE);
        buf.insert(&start);
        assert_eq!(buf.to_string(), start);

        buf.insert(".");
        assert_eq!(buf.to_string(), start + ".");
    }

    #[test]
    fn test_insert_with_grow_in_middle() {
        let mut buf = GapBuffer::new();
        let start = "a".repeat(32);
        buf.insert(&start);
        assert_eq!(buf.to_string(), start);

        let more = "b".repeat(DEFAULT_GAP_SIZE);
        buf.insert_at(&more, 16).expect("Should work");
        assert_eq!(buf.to_string(), "a".repeat(16) + &more + &"a".repeat(16));
    }

    #[test]
    fn test_basic_delete() {
        let mut buf = GapBuffer::new();
        let hello = String::from("Hello, world");
        buf.insert(&hello);
        buf.delete().expect("Should work.");

        assert_eq!(buf.to_string(), "Hello, worl");

        buf.delete().expect("Should work.");
        assert_eq!(buf.to_string(), "Hello, wor");
    }

    #[test]
    fn test_get_idx_with_empty_buffer() {
        let buf = GapBuffer::new();

        // If you try to index at gap_start, the valid char is really the one at gap_end
        assert_eq!(buf.translate_index(0), buf.gap_end);
        assert_eq!(buf.translate_index(buf.gap_start), buf.gap_end);

        // If you try to index after the gap, it should return your index + the gap size
        assert_eq!(
            buf.translate_index(buf.gap_end),
            buf.gap_size() + buf.gap_end
        );
        assert_eq!(
            buf.translate_index(buf.gap_end + 5),
            buf.gap_size() + buf.gap_end + 5
        );
    }

    #[test]
    fn test_get_idx_with_gap_at_end() {
        let mut buf = GapBuffer::new();
        buf.insert("Hello, world.");

        // A valid indices before the gap
        assert_eq!(buf.translate_index(0), 0);
        assert_eq!(buf.translate_index(3), 3);
        assert_eq!(buf.translate_index(buf.gap_start), buf.gap_end);

        // If you try to index after the gap, it should return your index + the gap size
        assert_eq!(
            buf.translate_index(buf.gap_end),
            buf.gap_size() + buf.gap_end
        );
        assert_eq!(
            buf.translate_index(buf.gap_end + 5),
            buf.gap_size() + buf.gap_end + 5
        );
    }

    #[test]
    fn test_get_idx_with_gap_in_middle() {
        let mut buf = GapBuffer::new();
        buf.insert("Hello, world.");
        buf.move_cursor(6).expect("Should work");

        // A valid indices before the gap
        assert_eq!(buf.translate_index(0), 0);
        assert_eq!(buf.translate_index(3), 3);
        assert_eq!(buf.translate_index(buf.gap_start), buf.gap_end);

        // A valid index after the gap
        assert_eq!(buf.translate_index(8), buf.gap_size() + 8);
        assert_eq!(
            buf.translate_index(buf.gap_end),
            buf.gap_size() + buf.gap_end
        );

        // invalid index
        assert_eq!(
            buf.translate_index(buf.gap_end + 20),
            buf.gap_size() + buf.gap_end + 20
        );
    }

    #[test]
    fn test_get_idx_with_gap_at_start() {
        let mut buf = GapBuffer::new();
        buf.insert("Hello, world.");
        buf.move_cursor(0).expect("Should work");

        assert_eq!(buf.translate_index(0), buf.gap_end);
        assert_eq!(buf.translate_index(buf.gap_start), buf.gap_end);
        assert_eq!(buf.translate_index(3), buf.gap_size() + 3);

        // A valid index after the gap
        assert_eq!(buf.translate_index(8), buf.gap_size() + 8);
        assert_eq!(
            buf.translate_index(buf.gap_end),
            buf.gap_size() + buf.gap_end
        );

        // invalid index
        assert_eq!(
            buf.translate_index(buf.gap_end + 20),
            buf.gap_size() + buf.gap_end + 20
        );
    }

    #[test]
    fn test_get() {
        let mut buf = GapBuffer::new();
        let hello = String::from("Hello, world");
        buf.insert(&hello);

        assert_eq!(buf.get(5), Some(&','));
        assert_eq!(buf.get(buf.gap_start), None);
        assert_eq!(buf.get(buf.gap_end), None);

        buf.move_cursor(5).expect("Should work");
        // Valid idx before the gap
        assert_eq!(buf.get(1), Some(&'e'));

        // valid idx (gap start)
        assert_eq!(buf.get(buf.gap_start), Some(&','));

        // valid idx after the gap
        assert_eq!(buf.get(7), Some(&'w'));

        // invalid idx after the gap
        assert_eq!(buf.get(buf.gap_end), None);
        assert_eq!(buf.get(30), None);
    }

    // TODO: need to add tests for iter(), and GapBufferIter
}
