use crate::char_iter;
use crate::gap_buf;
use std::iter::Rev;
use std::ops::{Bound, RangeBounds};

#[derive(Debug, Clone)]
pub struct GapBufferSlice<'a> {
    buff: &'a gap_buf::GapBuffer,
    start_index: usize,
    stop_index: usize,
}

impl<'a> GapBufferSlice<'a> {
    pub fn new(buff: &'a gap_buf::GapBuffer, start: usize, stop: usize) -> GapBufferSlice<'a> {
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

    pub fn start(&self) -> usize {
        self.start_index
    }

    pub fn chars(&self) -> char_iter::GapBufferChars<'a> {
        char_iter::GapBufferChars::new(self.clone(), 0, self.len())
    }

    pub fn chars_at(&self, index: usize) -> char_iter::GapBufferChars<'a> {
        if index >= self.len() {
            panic!(
                "Attempt to index past end of gap buffer slice. Buffer len: {}, index: {}.",
                self.len(),
                index
            );
        }

        char_iter::GapBufferChars::new(self.clone(), index, self.len())
    }

    pub fn chars_at_rev(&self, index: usize) -> Rev<char_iter::GapBufferChars<'a>> {
        if index >= self.len() {
            panic!(
                "Attempt to index past end of gap buffer slice. Buffer len: {}, index: {}.",
                self.len(),
                index
            );
        }

        char_iter::GapBufferChars::new(self.clone(), self.start_index, index + 1).rev()
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

#[cfg(test)]
mod tests {
    use crate::gap_buf::GapBuffer;
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
}
