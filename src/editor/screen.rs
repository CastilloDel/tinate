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

const WELCOME_MESSAGE: &'static str = "Tinate Is Not A Text Editor";

impl Editor {
    pub(super) fn refresh_screen(&mut self) -> Result<()> {
        self.draw_rows()?;

        io::stdout().flush()?;
        Ok(())
    }

    fn draw_rows(&mut self) -> Result<()> {
        let mut s = String::new();
        let (n_cols, n_rows) = term_size()?;
        let (n_cols, n_rows) = (n_cols as usize, n_rows as usize);
        queue!(s, MoveTo(0, 0))?;
        let mut rows_written = 0;
        let mut index = self.y_scroll;
        while rows_written < n_rows - 1 && index < self.buffer.len() {
            let mut part_number = 0;
            while rows_written < n_rows && part_number <= self.buffer[index].len() / n_cols {
                queue!(s, Clear(ClearType::CurrentLine))?;
                write!(
                    &mut s,
                    "{}\r\n",
                    self.buffer[index].take_substr(part_number * n_cols, n_cols)
                )?;
                rows_written += 1;
                part_number += 1;
            }
            index += 1;
        }
        while rows_written < n_rows - 1 {
            queue!(s, Clear(ClearType::CurrentLine))?;
            if self.file_name == "" && rows_written == n_rows / 3 {
                Editor::add_welcome_message(&mut s, n_cols)?;
            } else {
                write!(&mut s, "~\r\n")?;
            }
            rows_written += 1;
        }
        queue!(s, Clear(ClearType::CurrentLine))?;
        self.draw_status_bar(&mut s, n_cols)?;
        let cursor_screen_pos = self.cursor_pos_to_screen_pos(
            n_cols as u16,
            if self.mode == Mode::Insert {
                false
            } else {
                true
            },
        );
        queue!(s, MoveTo(cursor_screen_pos.0, cursor_screen_pos.1))?;
        print!("{}", s);
        Ok(())
    }

    fn add_welcome_message(s: &mut String, n_cols: usize) -> std::fmt::Result {
        let mut msg = String::from(WELCOME_MESSAGE);
        if WELCOME_MESSAGE.len() > n_cols as usize {
            msg.truncate(n_cols as usize);
        } else {
            Editor::write_padding(s, n_cols)?;
        }
        write!(s, "{}\r\n", msg)
    }

    fn write_padding(s: &mut String, n_cols: usize) -> std::fmt::Result {
        let padding = (n_cols as usize - WELCOME_MESSAGE.len()) / 2;
        if padding > 0 {
            write!(s, "~")?;
            let mut space = String::with_capacity(padding - 1);
            for _i in 0..padding - 1 {
                space.push(' ');
            }
            write!(s, "{}", space)?;
        }
        Ok(())
    }

    fn draw_status_bar(&self, s: &mut String, n_cols: usize) -> Result<()> {
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
        write!(s, "{}", bar.negative())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_bar_not_panic_with_little_windows() -> Result<()> {
        let editor = Editor::new();
        let mut s = String::new();
        editor.draw_status_bar(&mut s, 0)
    }
}
