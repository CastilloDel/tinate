use crossterm::terminal::size as term_size;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent},
    execute, queue,
    style::Styler,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    Result,
};
use std::cmp::min;
use std::env;
use std::fmt::Write as fmt_write;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process::exit;
mod modes;
use modes::Mode;
mod line;
use line::Line;
mod cursor;
use cursor::Cursor;

const WELCOME_MESSAGE: &'static str = "Tinate Is Not A Text Editor";

fn main() -> Result<()> {
    Editor::init()
}

struct Editor {
    buffer: Vec<Line>,
    cursor: Cursor,
    y_scroll: usize,
    file_name: String,
    mode: Mode,
    command_buffer: String,
}

impl Editor {
    pub fn init() -> Result<()> {
        execute!(io::stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;

        let mut editor = Editor::new();
        let args: Vec<String> = env::args().collect();
        if args.len() >= 2 {
            editor.load_to_buf(&args[1]).expect(
                "Invalid path or file. Keep in mind that tinate can only read Unicode valid files",
            );
        }
        loop {
            editor.refresh_screen()?;
            editor.process_event()?;
        }
    }

    fn new() -> Self {
        Editor {
            buffer: Vec::new(),
            cursor: Cursor::new(),
            y_scroll: 0,
            file_name: String::new(),
            mode: Mode::Normal,
            command_buffer: String::new(),
        }
    }

    fn load_to_buf(&mut self, path: &str) -> io::Result<()> {
        self.file_name = path.to_owned();
        match File::open(&self.file_name) {
            Ok(file) => {
                self.buffer = io::BufReader::new(file)
                    .lines()
                    .map(|line_result| line_result.map(|line| Line::new(&line)))
                    .collect::<io::Result<Vec<Line>>>()?;
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
        Ok(())
    }

    fn process_event(&mut self) -> Result<()> {
        let event = read()?;

        match self.mode {
            Mode::Normal => match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(':'),
                    ..
                }) => {
                    self.mode = Mode::Command;
                    self.command_buffer = String::new();
                    self.command_buffer.push(':');
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('h'),
                    ..
                }) => {
                    self.move_cursor_left(1, true);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    ..
                }) => {
                    self.move_cursor_down(1);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    ..
                }) => {
                    self.move_cursor_up(1);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('l'),
                    ..
                }) => {
                    self.move_cursor_right(1, true);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('i'),
                    ..
                }) => {
                    self.mode = Mode::Insert;
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    ..
                }) => {
                    self.move_cursor_right(1, false);
                    self.mode = Mode::Insert;
                }
                _ => {}
            },
            Mode::Command => match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(key),
                    ..
                }) => {
                    self.command_buffer.push(key);
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => self.execute_command()?,
                _ => {}
            },
            Mode::Insert => match event {
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
            },
        }
        Ok(())
    }

    fn refresh_screen(&mut self) -> Result<()> {
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

    fn execute_command(&mut self) -> Result<()> {
        match self.command_buffer.as_ref() {
            ":q" => {
                execute!(io::stdout(), LeaveAlternateScreen)?;
                //exit won't call destructors
                disable_raw_mode()?;
                exit(0);
            }
            ":w" => {
                self.mode = Mode::Normal;
                self.save_buffer()
            }
            ":wq" => {
                self.save_buffer()?;
                execute!(io::stdout(), LeaveAlternateScreen)?;
                //exit won't call destructors
                disable_raw_mode()?;
                exit(0);
            }
            _ => {
                self.mode = Mode::Normal;
                Ok(())
            }
        }
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
            self.move_cursor_left(1, true);
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
        let x = self.x(true);
        let y = self.y();
        self.buffer[y].remove(x);
    }

    fn save_buffer(&self) -> Result<()> {
        let mut file = File::create(&self.file_name)?;
        for line in self.buffer.iter() {
            file.write(line.get_content().as_bytes())?;
            file.write("\n".as_bytes())?;
        }
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
