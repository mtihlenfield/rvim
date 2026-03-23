use crate::slice;

pub struct GapBufferChars<'a> {
    buff: slice::GapBufferSlice<'a>,
    left_index: usize,
    right_index: usize,
}

impl<'a> GapBufferChars<'a> {
    pub fn new(
        buff: slice::GapBufferSlice<'a>,
        left_index: usize,
        right_index: usize,
    ) -> GapBufferChars<'a> {
        GapBufferChars {
            buff: buff,
            left_index: left_index,
            right_index: right_index,
        }
    }
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
    use crate::gap_buf;

    #[test]
    fn test_chars() {
        let mut buf = gap_buf::GapBuffer::new();
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
        let mut buf = gap_buf::GapBuffer::new();
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
        let mut buf = gap_buf::GapBuffer::new();
        buf.insert("Hello, world");
        buf.chars_at(30);
    }

    #[test]
    fn test_chars_at_rev() {
        let mut buf = gap_buf::GapBuffer::new();
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
}
