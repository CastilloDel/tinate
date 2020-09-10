use super::Editor;

#[derive(Copy, Clone, Debug)]
pub struct Cursor {
    pub x: usize,
    pub y: usize,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor { x: 0, y: 0 }
    }
}

impl Editor {
    fn pos(&self) -> (usize, usize) {
        let cursor = self.cursor;
        self.bound((cursor.x, cursor.y), true)
    }

    fn x(&self) -> usize {
        self.pos().0
    }

    fn y(&self) -> usize {
        self.pos().1
    }

    fn bound(&self, (x, mut y): (usize, usize), tight: bool) -> (usize, usize) {
        y = if y >= self.buffer.len() {
            self.buffer.len() - 1
        } else {
            y
        };

        let len = self.buffer[y].len() + if tight { 0 } else { 1 };
        if x >= len {
            if len == 0 {
                (0, y)
            } else {
                (len - 1, y)
            }
        } else {
            (x, y)
        }
    }

    fn bound_x(&self, (x, y): (usize, usize), tight: bool) -> (usize, usize) {
        (self.bound((x, y), tight).0, y)
    }

    fn bound_y(&self, (x, y): (usize, usize)) -> (usize, usize) {
        if y > self.buffer.len() - 1 {
            (x, self.buffer.len() - 1)
        } else {
            (x, y)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;

    #[test]
    fn bound_test() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("átaro"));
        assert_eq!(editor.bound((7, 1), true), (4, 0));
    }
}
