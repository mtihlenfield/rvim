use crate::slice;

pub struct GapBufferLines<'a> {
    buff: slice::GapBufferSlice<'a>,
    // Char index, not line index
    left_index: usize,
    right_index: usize,
    leading_newline: bool,
    trailing_newline: bool,
    single_char_done: bool,
}

impl<'a> GapBufferLines<'a> {
    pub fn new(
        buff: slice::GapBufferSlice<'a>,
        left_index: usize,
        right_index: usize,
    ) -> GapBufferLines<'a> {
        GapBufferLines {
            buff: buff,
            left_index: left_index,
            right_index: right_index,
            leading_newline: false,
            trailing_newline: false,
            single_char_done: false,
        }
    }
}

impl<'a> Iterator for GapBufferLines<'a> {
    type Item = slice::GapBufferSlice<'a>;

    fn next(&mut self) -> Option<slice::GapBufferSlice<'a>> {
        if self.trailing_newline {
            self.trailing_newline = false;
            return Some(self.buff.slice(self.left_index..self.left_index));
        }

        if self.left_index >= self.right_index {
            return None;
        }

        let mut chars = self
            .buff
            .chars_at(self.left_index)
            .take(self.right_index - self.left_index);

        if let Some(offset) = chars.position(|ch| ch == '\n') {
            let slice = self.buff.slice(self.left_index..=self.left_index + offset);
            self.left_index += offset + 1;

            if self.left_index == self.right_index {
                self.trailing_newline = true;
            }
            return Some(slice);
        }

        let slice = self.buff.slice(self.left_index..self.right_index);
        self.left_index = self.right_index;

        Some(slice)
    }
}

impl<'a> DoubleEndedIterator for GapBufferLines<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.leading_newline {
            self.leading_newline = false;
            return Some(self.buff.slice(self.right_index..=self.right_index));
        }

        if self.left_index > self.right_index || self.single_char_done {
            return None;
        } else if self.left_index == self.right_index && self.buff.len() == 1 {
            self.single_char_done = true;

            if self.buff.get(0) == Some('\n') {
                self.leading_newline = true;
                return Some(self.buff.slice(self.right_index..self.right_index));
            } else {
                return Some(self.buff.slice(self.right_index..=self.right_index));
            }
        } else if self.left_index == self.right_index {
            return None;
        }

        let chars = self
            .buff
            .chars_at_rev(self.right_index)
            .take(self.right_index - self.left_index + 1);

        for (offset, ch) in chars.enumerate() {
            if ch == '\n' {
                if offset == 0 {
                    if !self.trailing_newline && self.right_index == self.buff.len() - 1 {
                        self.trailing_newline = true;
                        return Some(self.buff.slice(self.right_index..self.right_index));
                    } else {
                        continue;
                    }
                }

                let left_index = self.right_index - offset + 1;
                let slice = self.buff.slice(left_index..=self.right_index);
                self.right_index -= offset;

                if self.right_index == self.left_index
                    && self.buff.get(self.right_index) == Some('\n')
                {
                    self.leading_newline = true;
                }

                return Some(slice);
            }
        }

        let slice = self.buff.slice(self.left_index..=self.right_index);
        self.right_index = self.left_index;

        Some(slice)
    }
}

#[cfg(test)]
mod tests {
    use crate::gap_buf;

    // #[test]
    // fn test_line_iter() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("hello\nworld");
    //     let mut lines_iter = buf.lines_at_char(0);

    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "hello\n"
    //     );
    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "world"
    //     );
    //     assert!(lines_iter.next().is_none());
    // }

    #[test]
    fn test_line_iter_rev() {
        let mut buf = gap_buf::GapBuffer::new();
        buf.insert("hello\nworld");
        let mut lines_iter = buf.lines_at_char_rev(buf.len() - 1);

        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "world"
        );
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello\n"
        );
        assert!(lines_iter.next().is_none());
    }

    // #[test]
    // fn test_line_iter_one_line() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("hello");
    //     let mut lines_iter = buf.lines_at_char(0);

    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "hello"
    //     );
    //     assert!(lines_iter.next().is_none());
    // }

    #[test]
    fn test_line_iter_one_line_rev() {
        let mut buf = gap_buf::GapBuffer::new();
        buf.insert("hello");
        let mut lines_iter = buf.lines_at_char_rev(buf.len() - 1);

        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert!(lines_iter.next().is_none());
    }

    // #[test]
    // fn test_line_iter_trailing_lines() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("hello\n\n\n");
    //     let mut lines_iter = buf.lines_at_char(0);

    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "hello\n"
    //     );
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert!(lines_iter.next().is_none());

    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("\n\n");
    //     let mut lines_iter = buf.lines_at_char(0);
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert!(lines_iter.next().is_none());
    // }

    #[test]
    fn test_line_iter_trailing_lines_rev() {
        let mut buf = gap_buf::GapBuffer::new();
        buf.insert("hello\n\n\n");
        let mut lines_iter = buf.lines_at_char_rev(buf.len() - 1);

        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello\n"
        );
        assert!(lines_iter.next().is_none());

        let mut buf = gap_buf::GapBuffer::new();
        buf.insert("\n\n");
        let mut lines_iter = buf.lines_at_char_rev(buf.len() - 1);
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
        assert!(lines_iter.next().is_none());
    }

    // #[test]
    // fn test_line_iter_many_lines() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("hello\nworld\ntest\n");
    //     let mut lines_iter = buf.lines_at_char(0);

    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "hello\n"
    //     );
    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "world\n"
    //     );
    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "test\n"
    //     );
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert!(lines_iter.next().is_none());
    // }

    #[test]
    fn test_line_iter_many_lines_rev() {
        let mut buf = gap_buf::GapBuffer::new();
        buf.insert("hello\nworld\ntest\n");
        let mut lines_iter = buf.lines_at_char_rev(buf.len() - 1);

        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "test\n"
        );
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "world\n"
        );
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello\n"
        );
        assert!(lines_iter.next().is_none());
    }

    // #[test]
    // fn test_line_iter_empty_buffer() {
    //     let buf = gap_buf::GapBuffer::new();
    //     let mut lines_iter = buf.lines_at_char(0);
    //     assert!(lines_iter.next().is_none());
    // }

    #[test]
    fn test_line_iter_empty_buffer_rev() {
        let buf = gap_buf::GapBuffer::new();
        let mut lines_iter = buf.lines_at_char_rev(0);
        assert!(lines_iter.next().is_none());
    }

    // #[test]
    // fn test_line_iter_single_newline() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("\n");
    //     let mut lines_iter = buf.lines_at_char(0);
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert!(lines_iter.next().is_none());
    // }

    #[test]
    fn test_line_iter_single_newline_rev() {
        let mut buf = gap_buf::GapBuffer::new();
        buf.insert("\n");
        let mut lines_iter = buf.lines_at_char_rev(buf.len() - 1);
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
        assert!(lines_iter.next().is_none());
    }

    // #[test]
    // fn test_line_iter_single_char() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("a");
    //     let mut lines_iter = buf.lines_at_char(0);
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "a");
    //     assert!(lines_iter.next().is_none());
    // }

    #[test]
    fn test_line_iter_single_char_rev() {
        let mut buf = gap_buf::GapBuffer::new();
        buf.insert("a");
        let mut lines_iter = buf.lines_at_char_rev(0);
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "a");
        assert!(lines_iter.next().is_none());
    }

    // #[test]
    // fn test_line_iter_leading_newline() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("\nhello");
    //     let mut lines_iter = buf.lines_at_char(0);
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "hello"
    //     );
    //     assert!(lines_iter.next().is_none());
    // }

    #[test]
    fn test_line_iter_leading_newline_rev() {
        let mut buf = gap_buf::GapBuffer::new();
        buf.insert("\nhello");
        let mut lines_iter = buf.lines_at_char_rev(buf.len() - 1);
        assert_eq!(
            lines_iter.next().unwrap().chars().collect::<String>(),
            "hello"
        );
        assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "\n");
        assert!(lines_iter.next().is_none());
    }

    // #[test]
    // fn test_lines_at() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("hello\nworld\ngoodbye");

    //     let mut iter = buf.lines_at(0).expect("should work");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "hello");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "world");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "goodbye");
    //     assert!(iter.next().is_none());

    //     let mut iter = buf.lines_at(1).expect("should work");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "world");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "goodbye");
    //     assert!(iter.next().is_none());

    //     let mut iter = buf.lines_at(2).expect("should work");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "goodbye");
    //     assert!(iter.next().is_none());
    // }

    // #[test]
    // fn test_lines_at_past_end() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("hello\nworld\ngoodbye");

    //     assert!(buf.lines_at(3).is_none());
    // }

    // #[test]
    // fn test_lines_at_empty_buffer() {
    //     let buf = gap_buf::GapBuffer::new();
    //     let iter = buf.lines_at(0);
    //     assert!(iter.is_none());
    // }

    // #[test]
    // fn test_lines_at_empty_lines() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("\n\nhello\n");
    //     let mut lines_iter = buf.lines_at(0).expect("should work");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "hello"
    //     );
    //     assert!(lines_iter.next().is_none());

    //     let mut lines_iter = buf.lines_at(1).expect("should work");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "hello"
    //     );
    //     assert!(lines_iter.next().is_none());

    //     let mut lines_iter = buf.lines_at(2).expect("should work");
    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "hello"
    //     );
    //     assert!(lines_iter.next().is_none());
    // }

    // #[test]
    // fn test_lines_at_rev() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("hello\nworld\ngoodbye");

    //     let mut iter = buf.lines_at_rev(2).expect("should work");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "goodbye");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "world");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "hello");
    //     assert!(iter.next().is_none());

    //     let mut iter = buf.lines_at_rev(1).expect("should work");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "world");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "hello");
    //     assert!(iter.next().is_none());

    //     let mut iter = buf.lines_at_rev(0).expect("should work");
    //     assert_eq!(iter.next().unwrap().chars().collect::<String>(), "hello");
    //     assert!(iter.next().is_none());
    // }

    // #[test]
    // fn test_lines_at_rev_past_end() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("hello\nworld\ngoodbye");

    //     assert!(buf.lines_at_rev(3).is_none());
    // }

    // #[test]
    // fn test_lines_at_rev_empty_buffer() {
    //     let buf = gap_buf::GapBuffer::new();
    //     let iter = buf.lines_at_rev(0);
    //     assert!(iter.is_none());
    // }

    // #[test]
    // fn test_lines_at_rev_empty_lines() {
    //     let mut buf = gap_buf::GapBuffer::new();
    //     buf.insert("\n\nhello\n");
    //     let mut lines_iter = buf.lines_at_rev(3).expect("should work");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "hello"
    //     );
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert!(lines_iter.next().is_none());

    //     let mut lines_iter = buf.lines_at_rev(2).expect("should work");
    //     assert_eq!(
    //         lines_iter.next().unwrap().chars().collect::<String>(),
    //         "hello"
    //     );
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert!(lines_iter.next().is_none());

    //     let mut lines_iter = buf.lines_at_rev(1).expect("should work");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert!(lines_iter.next().is_none());

    //     let mut lines_iter = buf.lines_at_rev(0).expect("should work");
    //     assert_eq!(lines_iter.next().unwrap().chars().collect::<String>(), "");
    //     assert!(lines_iter.next().is_none());
    // }
}
