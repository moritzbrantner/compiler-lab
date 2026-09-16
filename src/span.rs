use core::ops::Range;

/// A half-open UTF-8 byte range into the authoritative source buffer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Span {
    start: u32,
    end: u32,
}

impl Span {
    pub(crate) fn from_usize(start: usize, end: usize) -> Self {
        debug_assert!(start <= end);
        debug_assert!(end <= u32::MAX as usize);

        Self {
            start: start as u32,
            end: end as u32,
        }
    }

    #[must_use]
    pub const fn start(self) -> u32 {
        self.start
    }

    #[must_use]
    pub const fn end(self) -> u32 {
        self.end
    }

    #[must_use]
    pub const fn len(self) -> u32 {
        self.end - self.start
    }

    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    pub(crate) fn range(self) -> Range<usize> {
        self.start as usize..self.end as usize
    }
}
