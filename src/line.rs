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
}