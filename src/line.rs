pub const TAB_SZ: usize = 4;

use unicode_segmentation::UnicodeSegmentation;

pub struct Line {
    content: String,
    display: String,
}

impl Line {
    pub fn new(s: &str) -> Self {
        if s.contains('\n') {
            panic!("A Line can't contain a new line character('\n')")
        }
        let mut line = Line {
            content: s.to_owned(),
            display: String::new(),
        };
        line.update_display();
        line
    }

    pub fn len(&self) -> usize {
        self.display.graphemes(true).count()
    }

    pub fn truncate(&self, start: usize, max_len: usize) -> String {
        let start_index = match self.display.grapheme_indices(true).skip(start).next() {
            None => return String::from(""),
            Some((index, _)) => index,
        };
        match self
            .display
            .grapheme_indices(true)
            .skip(start + max_len)
            .next()
        {
            None => self.display[start_index..].to_owned(),
            Some((end_index, _)) => self.display[start_index..end_index].to_owned(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.content.is_empty()
    }

    pub fn is_valid_index(&self, index: usize) -> bool {
        let mut i = 0;
        let mut iter = self.content.graphemes(true);
        while i < index {
            match iter.next() {
                None => return false,
                Some("\t") => i += TAB_SZ - (i % TAB_SZ),
                Some(_) => i += 1,
            }
        }
        if i == index {
            true
        } else {
            false
        }
    }

    fn update_display(&mut self) {
        self.display.clear();
        let mut width = 0;
        for s in self.content.graphemes(true) {
            width += 1;
            if s == "\t" {
                self.display.push(' ');
                while width % TAB_SZ != 0 {
                    self.display.push(' ');
                    width += 1;
                }
            } else {
                self.display.push_str(s);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn new_line() {
        let line = super::Line::new("\taa\te");
        assert_eq!(line.display, "    aa  e");
    }

    #[test]
    fn new_line_not_ascii() {
        let line = super::Line::new("\táa\të");
        assert_eq!(line.display, "    áa  ë");
    }

    #[test]
    fn correct_len() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.len(), 9)
    }

    #[test]
    fn truncate_start() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.truncate(3, 20), " áñ  ë")
    }

    #[test]
    fn truncate_end() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.truncate(0, 6), "    áñ")
    }

    #[test]
    fn truncate_end_and_start() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.truncate(2, 6), "  áñ  ")
    }

    #[test]
    fn truncate_beyond_end() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.truncate(10, 13), "")
    }

    #[test]
    fn valid_index() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.is_valid_index(8), true);
    }

    #[test]
    fn invalid_index() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.is_valid_index(7), false);
    }
}
