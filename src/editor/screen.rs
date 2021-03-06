use super::{Editor, Mode};
use crossterm::terminal::size as term_size;
use crossterm::{
    cursor::MoveTo,
    queue,
    style::Styler,
    terminal::{Clear, ClearType},
    Result,
};
use std::cmp::min;
use std::fmt::Write as fmt_write;
use std::io;
use std::io::prelude::*;

impl Editor {
    pub(super) fn refresh_screen(&mut self) -> Result<()> {
        let mut buf = String::new();
        let term_size = term_size()?;
        self.recalculate_scroll(term_size);
        self.draw_rows(&mut buf, term_size)?;
        self.draw_status_bar(&mut buf, term_size.0)?;
        self.reposition_cursor(&mut buf, term_size.0)?;
        print!("{}", buf);
        io::stdout().flush()?;
        Ok(())
    }

    fn draw_rows(&mut self, buf: &mut String, term_size: (u16, u16)) -> Result<()> {
        let (n_cols, n_rows) = (term_size.0 as usize, term_size.1 as usize);
        queue!(buf, MoveTo(0, 0))?;
        let mut rows_written = 0;
        let mut index = self.y_scroll;
        while rows_written < n_rows - 1 && index < self.buffer.len() {
            let mut line_part = 0;
            while rows_written < n_rows && line_part <= self.buffer[index].len() / n_cols {
                queue!(buf, Clear(ClearType::CurrentLine))?;
                write!(
                    buf,
                    "{}\r\n",
                    self.buffer[index].take_substr(line_part * n_cols, n_cols)
                )?;
                rows_written += 1;
                line_part += 1;
            }
            index += 1;
        }
        while rows_written < n_rows - 1 {
            queue!(buf, Clear(ClearType::CurrentLine))?;
            write!(buf, "~\r\n")?;
            rows_written += 1;
        }
        Ok(())
    }

    fn draw_status_bar(&self, buf: &mut String, n_cols: u16) -> Result<()> {
        let n_cols = n_cols as usize;
        queue!(buf, Clear(ClearType::CurrentLine))?;
        let mut bar = String::new();
        if self.mode == Mode::Command {
            bar = self.command_buffer.clone();
        } else {
            write!(bar, "{} mode ", self.mode)?;
            write!(bar, "{}", self.file_name)?;
        }
        let row = self.y() + 1; //The stored pos is 0-indexed
        let row = String::from(" ") + &row.to_string();
        bar.truncate(n_cols - min(row.len(), n_cols));
        while n_cols - bar.len() > row.len() {
            write!(bar, " ")?;
        }
        write!(bar, "{}", row)?;
        write!(buf, "{}", bar.negative())?;
        Ok(())
    }

    fn reposition_cursor(&self, buf: &mut String, n_cols: u16) -> Result<()> {
        let cursor_screen_pos = self.cursor_pos_to_screen_pos(
            n_cols,
            if self.mode == Mode::Insert {
                false
            } else {
                true
            },
        );
        queue!(buf, MoveTo(cursor_screen_pos.0, cursor_screen_pos.1))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::super::Line;
    use super::*;

    #[test]
    fn status_bar_not_panic_with_little_windows() -> Result<()> {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new(""));
        let mut s = String::new();
        editor.draw_status_bar(&mut s, 0)
    }
}
