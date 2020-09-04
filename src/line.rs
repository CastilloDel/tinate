pub const TAB_SZ: usize = 4;

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
        for c in self.content.chars() {
            if c == '\t' {
                self.display.push(' ');
                while self.display.len() % TAB_SZ != 0 {
                    self.display.push(' ');
                }
            } else {
                self.display.push(c);
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
}
