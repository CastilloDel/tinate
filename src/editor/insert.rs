use super::{Editor, Mode};
use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    Result,
};

impl Editor {
    pub(super) fn match_event_insert(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                ..
            }) => self.insert_char(c)?,
            Event::Key(KeyEvent {
                code: KeyCode::Tab, ..
            }) => self.insert_char('\t')?,
            Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            }) => {
                self.mode = Mode::Normal;
                self.move_cursor_left(1, false);
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            }) => self.insert_new_line()?,
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => self.delete_back()?,
            _ => {}
        }
        Ok(())
    }

    fn insert_char(&mut self, c: char) -> Result<()> {
        let y = self.y();
        let x = self.x(false);
        self.buffer[y].insert(x, &c.to_string());
        self.move_cursor_right(1, false);
        Ok(())
    }

    fn insert_new_line(&mut self) -> Result<()> {
        let y = self.y();
        let x = self.x(false);
        let new_line = self.buffer[y].split_off(x);
        self.buffer.insert(self.y() + 1, new_line);
        self.cursor.x = 0;
        self.move_cursor_down(1);
        Ok(())
    }

    fn delete_back(&mut self) -> Result<()> {
        if self.x(false) != 0 {
            self.move_cursor_left(1, false);
            self.delete();
        } else if self.y() != 0 {
            self.move_cursor_up(1);
            let y = self.y();
            self.cursor.x = self.buffer[y].len();
            let remaining_line = self.buffer.remove(y + 1);
            self.buffer[y].push(&remaining_line.get_content());
        }
        Ok(())
    }

    fn delete(&mut self) {
        let x = self.x(false);
        let y = self.y();
        self.buffer[y].remove(x);
    }
}

#[cfg(test)]
mod tests {
    use super::super::{Cursor, Line};
    use super::*;
    #[test]
    fn insert_char() -> Result<()> {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("Frase"));
        editor.cursor.x = 5;
        editor.insert_char('1')?;
        assert_eq!(editor.buffer[0].get_content(), "Frase1");
        assert_eq!(editor.cursor.x, 6);
        Ok(())
    }

    #[test]
    fn newline() -> Result<()> {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("Frase"));
        editor.cursor.x = 2;
        editor.insert_new_line()?;
        assert_eq!(editor.buffer, vec![Line::new("Fr"), Line::new("ase")]);
        assert_eq!(editor.cursor, Cursor { x: 0, y: 1 });
        Ok(())
    }

    #[test]
    fn backspace() -> Result<()> {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("Frase"));
        editor.buffer.push(Line::new("1"));
        editor.cursor.y = 1;
        editor.delete_back()?;
        assert_eq!(editor.buffer[0].get_content(), "Frase1");
        assert_eq!(editor.cursor, Cursor { x: 5, y: 0 });
        Ok(())
    }
}
