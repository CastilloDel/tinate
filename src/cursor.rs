use super::Editor;
use std::cmp::min;

#[derive(Copy, Clone, Debug, PartialEq)]
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
    pub fn pos(&self, tight: bool) -> (usize, usize) {
        let cursor = self.cursor;
        self.bound((cursor.x, cursor.y), tight)
    }

    pub fn x(&self, tight: bool) -> usize {
        self.pos(tight).0
    }

    pub fn y(&self) -> usize {
        self.pos(true).1
    }

    fn bound(&self, (x, mut y): (usize, usize), tight: bool) -> (usize, usize) {
        if self.buffer.len() == 0 {
            return (0, 0);
        }
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

    pub fn move_cursor_right(&mut self, n: usize, tight: bool) {
        for _ in 0..n {
            match self.buffer[self.y()].next_valid_index(self.x(false)) {
                Some(index) => self.cursor.x = index,
                None => {
                    self.cursor.x = if tight {
                        self.buffer[self.y()].len() - min(self.buffer[self.y()].len(), 1)
                    } else {
                        self.buffer[self.y()].len()
                    };
                    return;
                }
            }
        }
    }

    pub fn move_cursor_left(&mut self, n: usize) {
        for _ in 0..n {
            self.cursor.x = match self.buffer[self.y()].prev_valid_index(self.x(false)) {
                Some(index) => index,
                None => return,
            }
        }
    }

    pub fn move_cursor_up(&mut self, n: usize) {
        for _ in 0..n {
            if self.y() == 0 {
                return;
            } else {
                self.cursor.y = self.y() - 1
            };
        }
        self.assert_valid_pos();
    }

    pub fn move_cursor_down(&mut self, n: usize) {
        for _ in 0..n {
            if self.y() == self.buffer.len() - 1 {
                return;
            } else {
                self.cursor.y = self.y() + 1
            };
        }
        self.assert_valid_pos();
    }

    fn assert_valid_pos(&mut self) {
        if !self.buffer[self.y()].is_valid_index(self.x(true)) {
            self.cursor.x = self.buffer[self.y()]
                .prev_valid_index(self.x(true))
                .unwrap_or(0);
        }
    }

    pub fn cursor_pos_to_screen_pos(&self, n_cols: u16, tight: bool) -> (u16, u16) {
        let (cursor_x, cursor_y) = self.pos(tight);
        let x = (cursor_x % n_cols as usize) as u16;
        let mut y = self.buffer[self.y_scroll..cursor_y]
            .iter()
            .fold(0, |acc, line| {
                acc + 1 + ((line.len() - min(1, line.len())) / n_cols as usize) as u16
            });
        y += (cursor_x / n_cols as usize) as u16;

        (x, y)
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

    #[test]
    fn move_right() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("átaro"));
        editor.move_cursor_right(4, true);
        assert_eq!(editor.cursor, Cursor { x: 4, y: 0 });
    }

    #[test]
    fn move_right_tabs() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("á\ttaro"));
        editor.move_cursor_right(4, true);
        assert_eq!(editor.cursor, Cursor { x: 6, y: 0 });
    }

    #[test]
    fn move_right_beyond_end() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("á\ttaro"));
        editor.move_cursor_right(10, false);
        assert_eq!(editor.cursor, Cursor { x: 8, y: 0 });
    }

    #[test]
    fn move_left_until_start() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("á\ttaro"));
        editor.cursor.x = 5;
        editor.move_cursor_left(10);
        assert_eq!(editor.cursor, Cursor { x: 0, y: 0 });
    }

    #[test]
    fn move_left() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("á\ttaro"));
        editor.cursor.x = 5;
        editor.move_cursor_left(1);
        assert_eq!(editor.cursor, Cursor { x: 4, y: 0 });
    }

    #[test]
    fn move_up() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("á\t\ttaro"));
        editor.buffer.push(Line::new("á\ttaro"));
        editor.cursor.x = 5;
        editor.cursor.y = 1;
        editor.move_cursor_up(1);
        assert_eq!(editor.cursor, Cursor { x: 4, y: 0 });
    }

    #[test]
    fn move_up_beyond_start() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("á\t\ttaro"));
        editor.buffer.push(Line::new("á\ttaro"));
        editor.cursor.y = 1;
        editor.move_cursor_up(10);
        assert_eq!(editor.cursor.y, 0);
    }

    #[test]
    fn move_down() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("áñëü\t"));
        editor.buffer.push(Line::new("á\tt"));
        editor.cursor.x = 3;
        editor.move_cursor_down(1);
        assert_eq!(editor.cursor, Cursor { x: 1, y: 1 });
    }

    #[test]
    fn move_down_beyond_end() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new(""));
        editor.buffer.push(Line::new(""));
        editor.move_cursor_down(10);
        assert_eq!(editor.cursor.y, 1);
    }

    #[test]
    fn screen_coords() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("ábcñ"));
        editor.buffer.push(Line::new("yerga"));
        editor.cursor.x = 4;
        editor.cursor.y = 1;
        assert_eq!(editor.cursor_pos_to_screen_pos(4, true), (0, 2));
    }

    #[test]
    fn screen_coords_2() {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("ábcñ"));
        editor.buffer.push(Line::new("yerga"));
        editor.cursor.x = 3;
        editor.cursor.y = 1;
        assert_eq!(editor.cursor_pos_to_screen_pos(4, true), (3, 1));
    }
}
