use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct Line {
    string: String,
}

impl Line {
    pub fn from(line_str: &str) -> Self {
        Self {
            string: String::from(line_str),
        }
    }

    pub fn get(&self, range: Range<usize>) -> String {
        let (start, end) = (range.start, range.end.min(self.len()));
        let graphemes = self.string.graphemes(true).collect::<Vec<&str>>();

        graphemes.get(start..end).unwrap_or_default().concat()
    }

    pub fn len(&self) -> usize {
        self.string.graphemes(true).count()
    }
}
