use std::cell::{Ref, RefCell};
use unicode_segmentation::UnicodeSegmentation;

/// Pre-cached line/column lookup table for a string slice.
pub struct LineColLookup<'source> {
    src: &'source str,
    line_heads: RefCell<Option<Vec<usize>>>,
}

impl<'source> LineColLookup<'source> {
    /// Creates a new line/col lookup table. The `src` parameter provides the input string used to calculate lines and columns.
    ///
    /// Internally, this scans `src` and caches the starting positions of all lines. This means this is an O(n) operation.
    pub fn new(src: &'source str) -> Self {
        Self {
            src,
            line_heads: RefCell::new(None),
        }
    }

    fn heads(&self) -> Ref<'_, Option<Vec<usize>>> {
        if self.line_heads.borrow().is_none() {
            let line_heads: Vec<usize> = std::iter::once(0)
                .chain(
                    self.src
                        .char_indices()
                        .filter_map(|(i, c)| Some(i + 1).filter(|_| c == '\n')),
                )
                .collect();
            self.line_heads.replace(Some(line_heads));
        }

        self.line_heads.borrow()
    }

    /// Looks up the 1-based line and column numbers of the specified byte index.
    ///
    /// Returns a tuple with the line number first, then column number.
    ///
    /// # Example
    ///
    /// let text = "One\nTwo";
    /// let lookup = LineColLookup::new(text);
    /// assert_eq!(lookup.get(0), (1, 1)); // 'O' (line 1, col 1)
    /// assert_eq!(lookup.get(1), (1, 2)); // 'n' (line 1, col 2)
    /// assert_eq!(lookup.get(2), (1, 3)); // 'e' (line 1, col 3)
    /// assert_eq!(lookup.get(4), (2, 1)); // 'T' (line 2, col 1)
    /// assert_eq!(lookup.get(5), (2, 2)); // 'w' (line 2, col 2)
    /// assert_eq!(lookup.get(6), (2, 3)); // 'o' (line 2, col 3)
    /// assert_eq!(lookup.get(7), (2, 4)); // <end> (line 2, col 4)
    ///
    ///
    /// # Panics
    ///
    /// Panics if `index` is greater than the length of the input `&str`.
    ///
    /// # Notes
    /// This function uses a binary search to locate the line on which `index` resides.
    /// This means that it runs in approximately O(log n) time.
    pub fn get(&self, index: usize) -> (usize, usize) {
        if index > self.src.len() {
            panic!("Index cannot be greater than the length of the input slice.");
        }

        if let Some(heads) = self.heads().as_ref() {
            // Perform a binary search to locate the line on which `index` resides
            let mut line_range = 0..heads.len();
            while line_range.end - line_range.start > 1 {
                let range_middle = line_range.start + (line_range.end - line_range.start) / 2;
                let (left, right) = (line_range.start..range_middle, range_middle..line_range.end);
                // Check which line window contains our character index
                if (heads[left.start]..heads[left.end]).contains(&index) {
                    line_range = left;
                } else {
                    line_range = right;
                }
            }

            let line_start_index = heads[line_range.start];
            let line = line_range.start + 1;
            let col = UnicodeSegmentation::graphemes(&self.src[line_start_index..index], true)
                .count()
                + 1;
            return (line, col);
        }

        unreachable!()
    }
}
