use crate::MAX_FRAME;

/// Collects bytes one at a time and hands back each complete frame.
///
/// A frame ends with a 0x00 byte. Frames that are too long are dropped, and
/// the reader starts again cleanly at the next 0x00.
pub struct FrameReader {
    buf: [u8; MAX_FRAME],
    len: usize,
    overflowed: bool,
}

impl FrameReader {
    pub const fn new() -> Self {
        FrameReader { buf: [0; MAX_FRAME], len: 0, overflowed: false }
    }

    /// Feed in one received byte. Returns the finished frame (including its
    /// final 0x00) when this byte completes one.
    pub fn push(&mut self, byte: u8) -> Option<&mut [u8]> {
        if self.len < self.buf.len() {
            self.buf[self.len] = byte;
            self.len += 1;
        } else {
            self.overflowed = true;
        }
        if byte != 0 {
            return None;
        }

        let len = self.len;
        let overflowed = self.overflowed;
        self.len = 0;
        self.overflowed = false;

        // A lone 0x00 is an empty frame, and an overflowed frame is unusable.
        if overflowed || len == 1 {
            None
        } else {
            Some(&mut self.buf[..len])
        }
    }
}

impl Default for FrameReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{vec, vec::Vec};

    /// Push every byte, collecting each frame that comes out.
    fn feed(reader: &mut FrameReader, bytes: &[u8]) -> Vec<Vec<u8>> {
        let mut frames = Vec::new();
        for &b in bytes {
            if let Some(frame) = reader.push(b) {
                frames.push(frame.to_vec());
            }
        }
        frames
    }

    #[test]
    fn frame_is_returned_only_when_complete() {
        let mut reader = FrameReader::new();
        assert!(feed(&mut reader, &[1, 2, 3]).is_empty());
        assert_eq!(feed(&mut reader, &[0]), vec![vec![1, 2, 3, 0]]);
    }

    #[test]
    fn back_to_back_frames_are_split() {
        let mut reader = FrameReader::new();
        assert_eq!(feed(&mut reader, &[1, 0, 2, 2, 0]), vec![vec![1, 0], vec![2, 2, 0]]);
    }

    #[test]
    fn empty_frames_are_ignored() {
        let mut reader = FrameReader::new();
        assert!(feed(&mut reader, &[0, 0, 0]).is_empty());
    }

    #[test]
    fn oversized_frame_is_dropped_and_the_next_one_works() {
        let mut reader = FrameReader::new();
        let mut bytes = vec![7u8; MAX_FRAME + 10];
        bytes.push(0);
        bytes.extend_from_slice(&[5, 0]);
        assert_eq!(feed(&mut reader, &bytes), vec![vec![5, 0]]);
    }

    #[test]
    fn frame_exactly_max_size_is_accepted() {
        let mut reader = FrameReader::new();
        let mut bytes = vec![9u8; MAX_FRAME - 1];
        bytes.push(0);
        assert_eq!(feed(&mut reader, &bytes).len(), 1);
    }
}
