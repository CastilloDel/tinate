use crossterm::terminal::size as term_size;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
    execute, queue,
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

const WELCOME_MESSAGE: &'static str = "Tinate Is Not A Text Editor";

fn main() -> Result<()> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    Editor::init()
}

struct Editor {
    buffer: Vec<String>,
    x_cursor_pos: u16,
    y_cursor_pos: u16,
    row_offset: usize,
    col_offset: usize,
}

impl Drop for Editor {
    fn drop(&mut self) {
        //The result isn't managed because it could cause a panic during a panic
        execute!(io::stdout(), EnterAlternateScreen).ok();
        disable_raw_mode().ok();
    }
}

impl Editor {
    pub fn init() -> Result<()> {
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

    fn new() -> Editor {
        Editor {
            buffer: Vec::new(),
            x_cursor_pos: 0,
            y_cursor_pos: 0,
            row_offset: 0,
            col_offset: 0,
        }
    }

    fn load_to_buf(&mut self, path: &str) -> io::Result<()> {
        let file = File::open(path)?;
        self.buffer = io::BufReader::new(file)
            .lines()
            .map(|line_result| line_result.map(|line| line.trim_end().to_string()))
            .collect::<io::Result<Vec<String>>>()?;
        Ok(())
    }

    fn process_event(&mut self) -> Result<()> {
        let event = read()?;

        match event {
            Event::Key(KeyEvent {
                modifiers: KeyModifiers::CONTROL,
                code: KeyCode::Char('q'),
            }) => {
                execute!(io::stdout(), LeaveAlternateScreen)?;
                //exit won't call destructors
                disable_raw_mode()?;
                exit(0);
            }
            Event::Key(key) if Editor::is_movement_key(&key) => self.move_cursor(key),
            _ => Ok(()),
        }
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
        for i in 0..n_rows {
            queue!(s, Clear(ClearType::CurrentLine))?;
            if i < self.buffer.len() - min(self.row_offset, self.buffer.len()) {
                let trunc_line =
                    Editor::trunc_line(&self.buffer[i + self.row_offset], n_cols, self.col_offset);
                if i == n_rows - 1 {
                    write!(&mut s, "{}\r", &trunc_line)?;
                } else {
                    write!(&mut s, "{}\r\n", &trunc_line)?;
                }
            } else {
                if self.buffer.len() == 0 && i == n_rows / 3 {
                    Editor::add_welcome_message(&mut s, n_cols)?;
                } else if i == n_rows - 1 {
                    write!(&mut s, "~")?;
                } else {
                    write!(&mut s, "~\r\n")?;
                }
            }
        }
        if self.buffer.len() != 0 {
            self.recalculate_cursor_pos();
        }
        queue!(s, MoveTo(self.x_cursor_pos, self.y_cursor_pos))?;
        print!("{}", s);
        Ok(())
    }
    fn trunc_line(line: &str, n_cols: usize, col_offset: usize) -> String {
        let mut trunc_line = line.to_owned();
        trunc_line.truncate(n_cols + col_offset);
        if col_offset > 0 {
            trunc_line = trunc_line.chars().rev().collect();
            trunc_line.truncate(trunc_line.len() - min(col_offset, trunc_line.len()));
            trunc_line = trunc_line.chars().rev().collect();
        }
        trunc_line
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

    fn recalculate_cursor_pos(&mut self) {
        let row_pos = self.y_cursor_pos as usize + self.row_offset; //position in the file
        if row_pos < self.buffer.len() {
            if self.x_cursor_pos as usize + self.col_offset > self.buffer[row_pos].len() {
                self.x_cursor_pos =
                    self.buffer[self.y_cursor_pos as usize + self.row_offset].len() as u16;
            }
        } else {
            if self.x_cursor_pos > 0 {
                self.x_cursor_pos = 0;
            }
        }
    }

    fn move_cursor(&mut self, key: KeyEvent) -> Result<()> {
        let (n_cols, n_rows) = term_size()?;
        match key.code {
            KeyCode::Char('h') => {
                if self.x_cursor_pos > 0 {
                    self.x_cursor_pos -= 1;
                } else {
                    if self.col_offset > 0 {
                        self.col_offset -= 1;
                    }
                    if self.x_cursor_pos > 0 {
                        self.x_cursor_pos = 0;
                    }
                }
            }
            KeyCode::Char('j') => {
                if self.y_cursor_pos < (n_rows - 1) {
                    self.y_cursor_pos += 1;
                } else {
                    self.row_offset += 1;
                }
            }
            KeyCode::Char('k') => {
                if self.y_cursor_pos > 0 {
                    self.y_cursor_pos -= 1;
                } else {
                    if self.row_offset > 0 {
                        self.row_offset -= 1;
                    }
                    if self.y_cursor_pos > 0 {
                        self.y_cursor_pos = 0;
                    }
                }
            }
            KeyCode::Char('l') => {
                if self.x_cursor_pos < (n_cols - 1) {
                    self.x_cursor_pos += 1;
                } else {
                    self.col_offset += 1;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn is_movement_key(key: &KeyEvent) -> bool {
        if key.modifiers != KeyModifiers::NONE {
            return false;
        }
        key.code == KeyCode::Char('h')
            || key.code == KeyCode::Char('j')
            || key.code == KeyCode::Char('k')
            || key.code == KeyCode::Char('l')
    }
}
