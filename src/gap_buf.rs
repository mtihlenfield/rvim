use std::iter::Rev;
use std::ops::{Bound, RangeBounds};

const DEFAULT_GAP_SIZE: usize = 64;

#[derive(Debug, Eq, PartialEq)]
pub enum GapBufferError {
    /// An attempt was made to move the cursor past the end of the GapBuffer
    MoveAfterEnd {
        buffer_len: usize,
        move_position: usize,
    },

    /// An attempt was made to delete backwards while cursor was at index 0
    DeleteFromStart,
}

impl std::fmt::Display for GapBufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MoveAfterEnd {
                buffer_len,
                move_position,
            } => {
                write!(
                    f,
                    "Attempted to move the cursor past the end of the buffer. Buffer len: {}, Move Position: {}",
                    buffer_len, move_position
                )
            }
            Self::DeleteFromStart => {
                write!(
                    f,
                    "Attempted to delete (backwards) while cursor was at index 0."
                )
            }
        }
    }
}

impl std::error::Error for GapBufferError {}

#[derive(Debug)]
pub struct GapBuffer {
    // TODO: using char means I'm using 4x memory than a u8... Probably should change it
    // eventually but it does simplify the interface and I think makes it faster since we can
    // index straight in to the char array instead of finding utf-8 boundaries.
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

    fn move_cursor(&mut self, pos: usize) -> Result<(), GapBufferError> {
        if pos > self.len() {
            return Err(GapBufferError::MoveAfterEnd {
                buffer_len: self.len(),
                move_position: pos,
            });
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

    fn grow_gap(&mut self, required_size: usize) {
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
    pub fn insert_at(&mut self, s: &str, pos: usize) -> Result<(), GapBufferError> {
        self.move_cursor(pos)?;
        self.insert(s);

        Ok(())
    }

    /// Delete at the current cursor position
    pub fn delete(&mut self) -> Result<(), GapBufferError> {
        if self.gap_start == 0 {
            return Err(GapBufferError::DeleteFromStart);
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

    pub fn get(&self, index: usize) -> Option<char> {
        let real_index = self.translate_index(index);
        self.buffer.get(real_index).copied()
    }

    pub fn get_coord(&self, row: usize, col: usize) -> Option<char> {
        let line = self.line_at(row)?;
        line.get(col)
    }

    pub fn slice<R>(&self, range: R) -> GapBufferSlice<'_>
    where
        R: RangeBounds<usize>,
    {
        let start_index = match range.start_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => *n + 1,
            Bound::Unbounded => 0,
        };

        let end_index = match range.end_bound() {
            Bound::Included(n) => *n + 1,
            Bound::Excluded(n) => *n,
            Bound::Unbounded => self.len(),
        };

        if end_index > self.len() {
            panic!("Attempt to slice with a range past end of buffer.");
        }

        GapBufferSlice::new(self, start_index, end_index)
    }

    fn find_line(&self, line_num: usize) -> Option<usize> {
        let mut line_count = 0;
        let mut char_count = 0;
        let mut chars = self.chars();

        while line_count < line_num {
            match chars.next() {
                Some('\n') => {
                    line_count += 1;
                    char_count += 1
                }
                Some(_) => char_count += 1,
                None => break,
            }
        }

        if line_count == line_num {
            Some(char_count)
        } else {
            None
        }
    }

    pub fn chars(&'_ self) -> GapBufferChars<'_> {
        GapBufferChars {
            buff: self.slice(..),
            left_index: 0,
            right_index: self.len(),
        }
    }

    pub fn chars_at(&'_ self, index: usize) -> GapBufferChars<'_> {
        if index >= self.len() {
            panic!(
                "Attempt to index past end of gap buffer. Buffer len: {}, index: {}.",
                self.len(),
                index
            );
        }

        GapBufferChars {
            buff: self.slice(..),
            left_index: index,
            right_index: self.len(),
        }
    }

    pub fn chars_at_rev(&'_ self, index: usize) -> Rev<GapBufferChars<'_>> {
        if index >= self.len() {
            panic!(
                "Attempt to index past end of gap buffer. Buffer len: {}, index: {}.",
                self.len(),
                index
            );
        }
        GapBufferChars {
            buff: self.slice(..),
            left_index: 0,
            right_index: index + 1, // +1 because exclusive end
        }
        .rev()
    }

    fn line_length(&self, start_index: usize) -> usize {
        (start_index..)
            .map(|i| self.buffer.get(i))
            .take_while(|ch| matches!(ch, Some(c) if **c != '\n'))
            .count()
    }

    pub fn line_at(&self, line: usize) -> Option<GapBufferSlice<'_>> {
        let start_index = self.find_line(line)?;
        Some(GapBufferSlice {
            buff: self,
            start_index: start_index,
            stop_index: start_index + self.line_length(start_index),
        })
    }

    pub fn lines_at(&self, line: usize) -> Option<GapBufferLines<'_>> {
        let line_index = self.find_line(line)?;
        Some(GapBufferLines {
            buff: self.slice(..),
            left_index: line_index,
            right_index: self.len(),
        })
    }

    pub fn lines_at_rev(&self, line: usize) -> Option<Rev<GapBufferLines<'_>>> {
        let buf_len = self.len();
        let line_start = self.find_line(line)?;
        // The `-1` covers to cases: gets rid of the trailing \n, or bumps down the index by one if
        // it's the end of the buffer.
        let line_end = if buf_len == line_start {
            buf_len
        } else {
            self.find_next(line_start, '\n').unwrap_or(self.len()) - 1
        };
        Some(
            GapBufferLines {
                buff: self.slice(..),
                left_index: 0,
                right_index: line_end,
            }
            .rev(),
        )
    }

    // /// Moving backwards from 'start', find the first instance of 'search_char'
    // /// and return it's index. The search range is inclusive: if the search_char is
    // /// found at 'start', start will be returned. Panics if 'start' is past the
    // /// end of the buffer.
    // pub fn find_prev(&self, start: usize, search_char: char) -> Option<usize> {
    //     for (index, c) in self.chars_at_rev(start).enumerate() {
    //         if c != search_char {
    //             continue;
    //         }

    //         return Some(start - index);
    //     }

    //     None
    // }

    /// Moving forwards from 'start', find the first instance of 'search_char'
    /// and return it's index. The search range is inclusive: if the search_char is
    /// found at 'start', start will be returned. Panics if 'start' is past the
    /// end of the buffer.
    pub fn find_next(&self, start: usize, search_char: char) -> Option<usize> {
        for (index, c) in self.chars_at(start).enumerate() {
            if c != search_char {
                continue;
            }

            return Some(start + index);
        }

        None
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

impl From<&str> for GapBuffer {
    fn from(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        let gap_size = DEFAULT_GAP_SIZE;
        let mut buffer = chars.clone();
        buffer.resize(chars.len() + gap_size, '\0');
        GapBuffer {
            buffer,
            gap_start: chars.len(),
            gap_end: chars.len() + gap_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GapBufferSlice<'a> {
    buff: &'a GapBuffer,
    start_index: usize,
    stop_index: usize,
}

impl<'a> GapBufferSlice<'a> {
    pub fn new(buff: &'a GapBuffer, start: usize, stop: usize) -> GapBufferSlice<'a> {
        GapBufferSlice {
            buff: buff,
            start_index: start,
            stop_index: stop,
        }
    }

    pub fn len(&self) -> usize {
        self.stop_index - self.start_index
    }

    pub fn get(&self, index: usize) -> Option<char> {
        let real_index = index + self.start_index;
        if real_index < self.stop_index {
            self.buff.get(real_index)
        } else {
            None
        }
    }

    pub fn chars(&self) -> GapBufferChars<'a> {
        GapBufferChars {
            buff: self.clone(),
            left_index: 0,
            right_index: self.len(),
        }
    }

    pub fn chars_at(&self, index: usize) -> GapBufferChars<'a> {
        if index >= self.len() {
            panic!(
                "Attempt to index past end of gap buffer slice. Buffer len: {}, index: {}.",
                self.len(),
                index
            );
        }

        GapBufferChars {
            buff: self.clone(),
            left_index: index,
            right_index: self.len(),
        }
    }

    pub fn chars_at_rev(&self, index: usize) -> Rev<GapBufferChars<'a>> {
        if index >= self.len() {
            panic!(
                "Attempt to index past end of gap buffer slice. Buffer len: {}, index: {}.",
                self.len(),
                index
            );
        }
        GapBufferChars {
            buff: self.slice(..),
            left_index: self.start_index,
            right_index: index + 1, // +1 because exclusive end
        }
        .rev()
    }

    pub fn slice<R>(&self, range: R) -> GapBufferSlice<'a>
    where
        R: RangeBounds<usize>,
    {
        // TODO: pull this out in to a function
        let start_index = match range.start_bound() {
            Bound::Included(n) => *n,
            Bound::Excluded(n) => *n + 1,
            Bound::Unbounded => 0,
        };

        let end_index = match range.end_bound() {
            Bound::Included(n) => *n + 1,
            Bound::Excluded(n) => *n,
            Bound::Unbounded => self.len(),
        };

        if end_index > self.len() {
            panic!("Attempt to slice with a range past end of slice.");
        }

        GapBufferSlice::new(
            self.buff,
            self.start_index + start_index,
            self.start_index + end_index,
        )
    }
}

pub struct GapBufferLines<'a> {
    buff: GapBufferSlice<'a>,
    // Char index, not line index
    left_index: usize,
    right_index: usize,
}

impl<'a> GapBufferLines<'a> {
    pub fn new(buff: GapBufferSlice<'a>) -> GapBufferLines<'a> {
        let buff_len = buff.len();
        GapBufferLines {
            buff: buff,
            left_index: 0,
            right_index: buff_len,
        }
    }
}

impl<'a> Iterator for GapBufferLines<'a> {
    type Item = GapBufferSlice<'a>;

    fn next(&mut self) -> Option<GapBufferSlice<'a>> {
        if self.left_index >= self.right_index {
            return None;
        }

        let max_chars = self.right_index - self.left_index;
        for (offset, ch) in self
            .buff
            .chars_at(self.left_index)
            .take(max_chars)
            .enumerate()
        {
            if ch != '\n' {
                continue;
            }

            let slice = self.buff.slice(self.left_index..self.left_index + offset);
            // Skip past the newline char
            self.left_index += offset + 1;
            return Some(slice);
        }

        let slice = self.buff.slice(self.left_index..);
        self.left_index = self.buff.len();

        Some(slice)
    }
}

impl<'a> DoubleEndedIterator for GapBufferLines<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.left_index >= self.right_index {
            return None;
        }

        let max_chars = self.right_index - self.left_index;
        for (offset, ch) in self
            .buff
            .chars_at_rev(self.right_index)
            .take(max_chars)
            .enumerate()
        {
            if ch != '\n' {
                continue;
            }

            let slice = self
                .buff
                .slice(self.right_index - offset + 1..=self.right_index);
            self.right_index -= offset + 1;
            return Some(slice);
        }

        let slice = self.buff.slice(..=self.right_index);
        self.right_index = 0;

        Some(slice)
    }
}

pub struct GapBufferChars<'a> {
    buff: GapBufferSlice<'a>,
    left_index: usize,
    right_index: usize,
}

impl<'a> Iterator for GapBufferChars<'a> {
    type Item = char;

    #[inline]
    fn next(&mut self) -> Option<char> {
        if self.left_index < self.right_index {
            let val = self.buff.get(self.left_index);
            self.left_index += 1;
            val
        } else {
            None
        }
    }
}

impl<'a> DoubleEndedIterator for GapBufferChars<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.left_index < self.right_index {
            self.right_index -= 1;
            let val = self.buff.get(self.right_index);
            val
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let buf = GapBuffer::default();
        assert_eq!(buf.len(), 0);
        assert_eq!(buf.gap_start, 0);
        assert_eq!(buf.gap_end, DEFAULT_GAP_SIZE);
    }

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
    fn test_move_cursor_equal() {
        let mut buf = GapBuffer::new();
        buf.insert("Hello, world");
        buf.move_cursor(5).expect("Should work");
        let gap_start = buf.gap_start;
        let gap_end = buf.gap_end;
        buf.move_cursor(5).expect("Should work");

        assert_eq!(buf.gap_start, gap_start);
        assert_eq!(buf.gap_end, gap_end);
    }

    #[test]
    fn test_move_cursor_invalid() {
        let mut buf = GapBuffer::new();
        match buf.move_cursor(1) {
            Ok(()) => panic!("Should have gotten error"),
            Err(GapBufferError::MoveAfterEnd {
                buffer_len,
                move_position,
            }) => {
                assert_eq!(buffer_len, 0);
                assert_eq!(move_position, 1);
            }
            Err(e) => panic!("{}", e),
        };

        buf.insert("Hello, world");
        match buf.move_cursor(20) {
            Ok(()) => panic!("Should have gotten error"),
            Err(GapBufferError::MoveAfterEnd {
                buffer_len,
                move_position,
            }) => {
                assert_eq!(buffer_len, 12);
                assert_eq!(move_position, 20);
            }
            Err(e) => panic!("{}", e),
        };
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
    fn test_from() {
        let s = "Hello, world";

        let buff: GapBuffer = s.into();
        assert_eq!(buff.chars().collect::<String>(), s);
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
    fn test_delete_from_start() {
        let mut buf = GapBuffer::new();
        assert_eq!(buf.delete().unwrap_err(), GapBufferError::DeleteFromStart);

        let hello = String::from("Hello, world");
        buf.insert(&hello);
        buf.move_cursor(0).expect("Should work.");
        assert_eq!(buf.delete().unwrap_err(), GapBufferError::DeleteFromStart);
    }

    #[test]
    fn test_get_index_with_empty_buffer() {
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
    fn test_get_index_with_gap_at_end() {
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
    fn test_get_index_with_gap_in_middle() {
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
    fn test_get_index_with_gap_at_start() {
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

        assert_eq!(buf.get(5), Some(','));
        assert_eq!(buf.get(buf.gap_start), None);
        assert_eq!(buf.get(buf.gap_end), None);

        buf.move_cursor(5).expect("Should work");
        // Valid index before the gap
        assert_eq!(buf.get(1), Some('e'));

        // valid index (gap start)
        assert_eq!(buf.get(buf.gap_start), Some(','));

        // valid index after the gap
        assert_eq!(buf.get(7), Some('w'));

        // invalid index after the gap
        assert_eq!(buf.get(buf.gap_end), None);
        assert_eq!(buf.get(30), None);
    }

    #[test]
    fn test_is_empty() {
        let mut buf = GapBuffer::new();
        assert_eq!(buf.is_empty(), true);

        buf.insert("Hello, world");
        assert_eq!(buf.is_empty(), false);
    }

    #[test]
    fn test_len() {
        let mut buf = GapBuffer::new();
        assert_eq!(buf.len(), 0);

        let hello = String::from("Hello, world");
        buf.insert(&hello);
        assert_eq!(buf.len(), hello.len());

        buf.move_cursor(0).expect("Should work");
        assert_eq!(buf.len(), hello.len());

        buf.move_cursor(hello.len()).expect("Should work");
        assert_eq!(buf.len(), hello.len());
    }

    #[test]
    fn test_chars() {
        let mut buf = GapBuffer::new();
        assert_eq!(buf.chars().collect::<Vec<_>>().len(), 0);

        let hello = "Hello, world";

        // test with gap at end
        buf.insert(hello);
        let new_str: String = buf.chars().collect();
        assert_eq!(new_str, hello);

        // test with gap in middle
        buf.move_cursor(5).expect("Should work");
        let new_str: String = buf.chars().collect();
        assert_eq!(new_str, hello);

        // test with gap at start
        buf.move_cursor(0).expect("Should work");
        let new_str: String = buf.chars().collect();
        assert_eq!(new_str, hello);
    }

    #[test]
    fn test_chars_at() {
        let mut buf = GapBuffer::new();
        let hello = "Hello, world";
        buf.insert(hello);

        let new_str: String = buf.chars_at(7).collect();
        assert_eq!(new_str, "world");
        let new_str: String = buf.chars_at(0).collect();
        assert_eq!(new_str, "Hello, world");

        // test with gap in middle
        buf.move_cursor(5).expect("Should work");
        let new_str: String = buf.chars_at(7).collect();
        assert_eq!(new_str, "world");
        let new_str: String = buf.chars_at(0).collect();
        assert_eq!(new_str, "Hello, world");

        // test with gap at start
        buf.move_cursor(0).expect("Should work");
        let new_str: String = buf.chars_at(7).collect();
        assert_eq!(new_str, "world");
        let new_str: String = buf.chars_at(0).collect();
        assert_eq!(new_str, "Hello, world");
    }

    #[test]
    #[should_panic]
    fn test_chars_at_panic() {
        let mut buf = GapBuffer::new();
        buf.insert("Hello, world");
        buf.chars_at(30);
    }

    #[test]
    fn test_chars_at_rev() {
        let mut buf = GapBuffer::new();
        let hello = "Hello, world";
        let olleh = "dlrow ,olleH";

        // test with gap at end
        buf.insert(hello);
        let new_str: String = buf.chars_at_rev(hello.len() - 1).collect();
        assert_eq!(new_str, olleh);

        let new_str: String = buf.chars_at_rev(0).collect();
        assert_eq!(new_str, "H");

        // test with gap in middle
        buf.move_cursor(5).expect("Should work");
        let new_str: String = buf.chars_at_rev(5).collect();
        assert_eq!(new_str, ",olleH");

        // test with gap at start
        buf.move_cursor(0).expect("Should work");
        let new_str: String = buf.chars_at_rev(5).collect();
        assert_eq!(new_str, ",olleH");
    }

    #[test]
    #[should_panic]
    fn test_find_next_empty_buffer() {
        let buf = GapBuffer::new();
        assert_eq!(buf.find_next(0, 'c'), None);
    }

    #[test]
    #[should_panic]
    fn test_find_next_bad_index() {
        let mut buf = GapBuffer::new();
        buf.insert("hello");
        assert_eq!(buf.find_next(10, 'c'), None);
    }

    #[test]
    fn test_find_next_with_match() {
        let mut buf = GapBuffer::new();
        buf.insert("hello");
        assert_eq!(buf.find_next(0, 'h'), Some(0));
        assert_eq!(buf.find_next(4, 'o'), Some(4));
        assert_eq!(buf.find_next(0, 'e'), Some(1));
        assert_eq!(buf.find_next(1, 'l'), Some(2));
        assert_eq!(buf.find_next(4, 'e'), None);
    }

    #[test]
    fn test_find_next_without_match() {
        let mut buf = GapBuffer::new();
        buf.insert("hello");
        assert_eq!(buf.find_next(0, 'x'), None);
        assert_eq!(buf.find_next(4, 'x'), None);
    }

    // #[test]
    // #[should_panic]
    // fn test_find_prev_empty_buffer() {
    //     let buf = GapBuffer::new();
    //     assert_eq!(buf.find_prev(0, 'c'), None);
    // }

    // #[test]
    // #[should_panic]
    // fn test_find_prev_bad_index() {
    //     let mut buf = GapBuffer::new();
    //     buf.insert("hello");
    //     assert_eq!(buf.find_prev(10, 'c'), None);
    // }

    // #[test]
    // fn test_find_prev_with_match() {
    //     let mut buf = GapBuffer::new();
    //     buf.insert("hello");
    //     assert_eq!(buf.find_prev(0, 'h'), Some(0));
    //     assert_eq!(buf.find_prev(4, 'o'), Some(4));
    //     assert_eq!(buf.find_prev(4, 'e'), Some(1));
    //     assert_eq!(buf.find_prev(3, 'e'), Some(1));
    //     assert_eq!(buf.find_prev(1, 'e'), Some(1));
    // }

    // #[test]
    // fn test_find_prev_without_match() {
    //     let mut buf = GapBuffer::new();
    //     buf.insert("hello");
    //     assert_eq!(buf.find_prev(0, 'x'), None);
    //     assert_eq!(buf.find_prev(4, 'x'), None);
    // }

    #[test]
    fn test_slice() {
        let mut buf = GapBuffer::new();
        buf.insert("hello");
        let slice = buf.slice(..);
        assert_eq!(slice.chars().collect::<String>(), "hello");

        let slice = buf.slice(0..4);
        assert_eq!(slice.chars().collect::<String>(), "hell");

        let slice = buf.slice(0..5);
        assert_eq!(slice.chars().collect::<String>(), "hello");

        let slice = buf.slice(0..0);
        assert_eq!(slice.chars().collect::<String>(), "");

        let slice = buf.slice(1..2);
        assert_eq!(slice.chars().collect::<String>(), "e");
    }

    #[test]
    #[should_panic]
    fn test_slice_past_end() {
        let mut buf = GapBuffer::new();
        buf.insert("hello");
        let _ = buf.slice(..20);
    }

    #[test]
    fn test_slice_of_slice() {
        let mut buf = GapBuffer::new();
        buf.insert("hello, world");
        let parent_slice = buf.slice(..);

        let child = parent_slice.slice(..5);
        assert_eq!(child.chars().collect::<String>(), "hello");

        let child = parent_slice.slice(7..);
        assert_eq!(child.chars().collect::<String>(), "world");

        let parent_slice = buf.slice(1..6);
        let child = parent_slice.slice(..);
        assert_eq!(child.chars().collect::<String>(), "ello,");

        let child = parent_slice.slice(1..2);
        assert_eq!(child.chars().collect::<String>(), "l");
    }

    #[test]
    #[should_panic]
    fn test_slice_of_slice_past_end() {
        let mut buf = GapBuffer::new();
        buf.insert("hello");
        let parent_slice = buf.slice(..);
        let _ = parent_slice.slice(..20);
    }

    #[test]
    fn test_line_iter() {
        let mut buf = GapBuffer::new();
        buf.insert("hello\nworld");
        let mut lines_iter = GapBufferLines::new(buf.slice(..));

        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "world"
        );
        assert!(lines_iter.next().is_none());
    }

    #[test]
    fn test_line_iter_one_line() {
        let mut buf = GapBuffer::new();
        buf.insert("hello");
        let mut lines_iter = GapBufferLines::new(buf.slice(..));

        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert!(lines_iter.next().is_none());
    }

    #[test]
    fn test_line_iter_many_lines() {
        let mut buf = GapBuffer::new();
        buf.insert("hello\nworld\ntest\n");
        let mut lines_iter = GapBufferLines::new(buf.slice(..));

        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "world"
        );
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "test"
        );
        assert!(lines_iter.next().is_none());
        assert!(lines_iter.next().is_none());
    }

    #[test]
    fn test_line_iter_empty_buffer() {
        let buf = GapBuffer::new();
        let mut lines_iter = GapBufferLines::new(buf.slice(..));
        assert!(lines_iter.next().is_none());
    }

    #[test]
    fn test_line_iter_empty_lines() {
        let mut buf = GapBuffer::new();
        buf.insert("\n\nhello\n");
        let mut lines_iter = GapBufferLines::new(buf.slice(..));
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert!(lines_iter.next().is_none());
    }

    #[test]
    fn test_lines_at() {
        let mut buf = GapBuffer::new();
        buf.insert("hello\nworld\ngoodbye");

        let mut iter = buf.lines_at(0).expect("should work");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "hello");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "world");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "goodbye");
        assert!(iter.next().is_none());

        let mut iter = buf.lines_at(1).expect("should work");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "world");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "goodbye");
        assert!(iter.next().is_none());

        let mut iter = buf.lines_at(2).expect("should work");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "goodbye");
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_lines_at_past_end() {
        let mut buf = GapBuffer::new();
        buf.insert("hello\nworld\ngoodbye");

        assert!(buf.lines_at(3).is_none());
    }

    #[test]
    fn test_lines_at_empty_buffer() {
        let buf = GapBuffer::new();
        let iter = buf.lines_at(0);
        assert!(iter.is_none());
    }

    #[test]
    fn test_lines_at_empty_lines() {
        let mut buf = GapBuffer::new();
        buf.insert("\n\nhello\n");
        let mut lines_iter = buf.lines_at(0).expect("should work");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert!(lines_iter.next().is_none());

        let mut lines_iter = buf.lines_at(1).expect("should work");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert!(lines_iter.next().is_none());

        let mut lines_iter = buf.lines_at(2).expect("should work");
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert!(lines_iter.next().is_none());
    }

    #[test]
    fn test_lines_at_rev() {
        let mut buf = GapBuffer::new();
        buf.insert("hello\nworld\ngoodbye");

        let mut iter = buf.lines_at_rev(2).expect("should work");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "goodbye");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "world");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "hello");
        assert!(iter.next().is_none());

        let mut iter = buf.lines_at_rev(1).expect("should work");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "world");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "hello");
        assert!(iter.next().is_none());

        let mut iter = buf.lines_at_rev(0).expect("should work");
        assert_eq!(iter.next().unwrap().chars().collect::<String>(), "hello");
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_lines_at_rev_past_end() {
        let mut buf = GapBuffer::new();
        buf.insert("hello\nworld\ngoodbye");

        assert!(buf.lines_at_rev(3).is_none());
    }

    #[test]
    fn test_lines_at_rev_empty_buffer() {
        let buf = GapBuffer::new();
        let iter = buf.lines_at_rev(0);
        assert!(iter.is_none());
    }

    #[test]
    fn test_lines_at_rev_empty_lines() {
        let mut buf = GapBuffer::new();
        buf.insert("\n\nhello\n");
        let mut lines_iter = buf.lines_at_rev(3).expect("should work");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert!(lines_iter.next().is_none());

        let mut lines_iter = buf.lines_at_rev(2).expect("should work");
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert!(lines_iter.next().is_none());

        let mut lines_iter = buf.lines_at_rev(1).expect("should work");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert!(lines_iter.next().is_none());

        let mut lines_iter = buf.lines_at_rev(0).expect("should work");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert!(lines_iter.next().is_none());
    }
}
