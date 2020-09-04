pub const TAB_SZ: usize = 4;

use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

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

    fn update_display(&mut self) {
        self.display.clear();
        for s in self.content.graphemes(true) {
            if s == "\t" {
                self.display.push(' ');
                while self.display.width() % TAB_SZ != 0 {
                    self.display.push(' ');
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
}
