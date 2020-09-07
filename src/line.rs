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

    pub fn take_substr(&self, start: usize, max_len: usize) -> String {
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
        if index >= self.len() {
            return false;
        }
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

    pub fn next_valid_index(&self, index: usize) -> Option<usize> {
        let mut i = 0;
        let mut iter = self.content.graphemes(true);
        while i <= index {
            match iter.next() {
                None => return None,
                Some("\t") => i += TAB_SZ - (i % TAB_SZ),
                Some(_) => i += 1,
            }
        }
        if let None = iter.next() {
            None
        } else {
            Some(i)
        }
    }

    pub fn prev_valid_index(&self, index: usize) -> Option<usize> {
        if index == 0 {
            return None;
        }
        let mut i = 0;
        let mut prev_i = 0;
        let mut iter = self.content.graphemes(true);
        while i < index {
            prev_i = i;
            match iter.next() {
                None => return None,
                Some("\t") => i += TAB_SZ - (i % TAB_SZ),
                Some(_) => i += 1,
            }
        }
        Some(prev_i)
    }

    pub fn insert(&mut self, index: usize, s: &str) {
        let mut i = 0;
        let mut iter = self.content.grapheme_indices(true);
        while i < index {
            match iter.next() {
                None => panic!("Line: Tried to insert in a invalid index({})", index),
                Some((_, grapheme)) => {
                    if grapheme == "\t" {
                        i += TAB_SZ - (i % TAB_SZ);
                    } else {
                        i += 1;
                    }
                }
            }
        }
        if i != index {
            panic!("Line: Tried to insert in a invalid index({})", index);
        }
        if let Some((content_index, _)) = iter.next() {
            self.content.insert_str(content_index, s);
        } else {
            self.content.insert_str(self.content.len(), s);
        }
        self.update_display();
    }

    pub fn split_off(&mut self, at: usize) -> Line {
        if at == self.len() {
            return Line::new("");
        }
        assert!(self.is_valid_index(at));
        let other = self.content.split_off(self.get_content_index(at));
        self.update_display();
        Line::new(&other)
    }

    fn get_content_index(&self, index: usize) -> usize {
        let mut i = 0;
        let mut iter = self.content.grapheme_indices(true);
        while i < index {
            match iter.next() {
                None => panic!("Line: Tried to translate an invalid index({})", index),
                Some((_, grapheme)) => {
                    if grapheme == "\t" {
                        i += TAB_SZ - (i % TAB_SZ);
                    } else {
                        i += 1;
                    }
                }
            }
        }
        if i != index {
            panic!("Line: Tried to translate an invalid index({})", index);
        }
        if let Some((content_index, _)) = iter.next() {
            content_index
        } else {
            panic!("Line: Tried to translate an invalid index({})", index);
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
    fn take_substr_start() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.take_substr(3, 20), " áñ  ë")
    }

    #[test]
    fn take_substr_end() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.take_substr(0, 6), "    áñ")
    }

    #[test]
    fn take_substr_end_and_start() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.take_substr(2, 6), "  áñ  ")
    }

    #[test]
    fn take_substr_beyond_end() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.take_substr(10, 13), "")
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

    #[test]
    fn invalid_index_beyond_len() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.is_valid_index(9), false);
    }

    #[test]
    fn next_index() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.next_valid_index(2), Some(4));
    }

    #[test]
    fn no_next_index() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.next_valid_index(8), None);
    }

    #[test]
    fn prev_index() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.prev_valid_index(8), Some(6));
    }

    #[test]
    fn no_prev_index() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.prev_valid_index(0), None);
    }

    #[test]
    fn insert_char() {
        let mut line = super::Line::new("\táñ\të");
        line.insert(5, "ö");
        assert_eq!(line.content, "\táöñ\të");
    }

    #[test]
    fn insert_char_one_beyond_len() {
        let mut line = super::Line::new("\táñ\të");
        line.insert(9, "ö");
        assert_eq!(line.content, "\táñ\tëö");
    }

    #[test]
    #[should_panic]
    fn insert_char_beyond_len() {
        let mut line = super::Line::new("\táñ\të");
        line.insert(10, "ö");
    }

    #[test]
    fn split_off_half() {
        let mut line = super::Line::new("\táñ\të");
        assert_eq!(line.split_off(5).content, "ñ\të");
    }

    #[test]
    fn split_off_start() {
        let mut line = super::Line::new("\táñ\të");
        assert_eq!(line.split_off(0).content, "\táñ\të");
    }

    #[test]
    fn split_off_end() {
        let mut line = super::Line::new("\táñ\të");
        assert_eq!(line.split_off(9).content, "");
    }

    #[test]
    fn content_index() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.get_content_index(5), 3);
    }

    #[test]
    fn content_index_zero() {
        let line = super::Line::new("\táñ\të");
        assert_eq!(line.get_content_index(0), 0);
    }

    #[test]
    #[should_panic]
    fn content_index_beyond_len() {
        let line = super::Line::new("\táñ\të");
        line.get_content_index(9);
    }
}
