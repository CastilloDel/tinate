use crossterm::terminal::size as term_size;
use crossterm::{
    cursor::MoveTo,
    event::{read, Event, KeyCode, KeyEvent, KeyModifiers},
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

const WELCOME_MESSAGE: &'static str = "Tinate Is Not A Text Editor";
const TAB_SZ: usize = 4;

fn main() -> Result<()> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;

    Editor::init()
}

struct Editor {
    buffer: Vec<String>,
    render_buffer: Vec<String>,
    x_cursor_pos: u16,
    y_cursor_pos: u16,
    row_offset: usize,
    col_offset: usize,
    file_name: String,
    mode: Mode,
    command_buffer: String,
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

    fn new() -> Self {
        Editor {
            buffer: Vec::new(),
            render_buffer: Vec::new(),
            x_cursor_pos: 0,
            y_cursor_pos: 0,
            row_offset: 0,
            col_offset: 0,
            file_name: String::new(),
            mode: Mode::Normal,
            command_buffer: String::new(),
        }
    }

    fn load_to_buf(&mut self, path: &str) -> io::Result<()> {
        self.file_name = path.to_owned();
        let file = File::open(&self.file_name)?;
        self.buffer = io::BufReader::new(file)
            .lines()
            .map(|line_result| line_result.map(|line| line.trim_end().to_string()))
            .collect::<io::Result<Vec<String>>>()?;
        self.update_render_buf();
        Ok(())
    }

    fn update_render_buf(&mut self) {
        for (index, line) in self.buffer.iter().enumerate() {
            self.render_buffer.push(String::new());
            for c in line.chars() {
                if c == '\t' {
                    self.render_buffer[index].push(' ');
                    while self.render_buffer[index].len() % TAB_SZ != 0 {
                        self.render_buffer[index].push(' ');
                    }
                } else {
                    self.render_buffer[index].push(c);
                }
            }
        }
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
                    Ok(())
                }
                Event::Key(key) if Editor::is_movement_key(&key) => self.move_cursor(key),
                _ => Ok(()),
            },
            Mode::Command => match event {
                Event::Key(KeyEvent {
                    code: KeyCode::Char(key),
                    ..
                }) => {
                    self.command_buffer.push(key);
                    Ok(())
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Enter,
                    ..
                }) => {
                    self.execute_command()?;
                    Ok(())
                }
                _ => Ok(()),
            },
            Mode::Insert => match event {
                _ => Ok(()),
            },
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
        for i in 0..n_rows - 1 {
            queue!(s, Clear(ClearType::CurrentLine))?;
            if i < self.render_buffer.len() - min(self.row_offset, self.render_buffer.len()) {
                let trunc_line = Editor::trunc_line(
                    &self.render_buffer[i + self.row_offset],
                    n_cols,
                    self.col_offset,
                );
                write!(&mut s, "{}\r\n", &trunc_line)?;
            } else {
                if self.render_buffer.len() == 0 && i == n_rows / 3 {
                    Editor::add_welcome_message(&mut s, n_cols)?;
                }
                write!(&mut s, "~\r\n")?;
            }
        }
        self.draw_status_bar(&mut s, n_cols)?;
        if self.render_buffer.len() != 0 {
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

    fn draw_status_bar(&self, s: &mut String, n_cols: usize) -> Result<()> {
        let mut bar = String::new();
        if self.mode == Mode::Command {
            bar = self.command_buffer.clone();
        } else {
            write!(bar, "{} mode ", self.mode)?;
            write!(bar, "{}", self.file_name)?;
        }
        let row = self.row_offset + self.y_cursor_pos as usize + 1;
        let row = String::from(" ") + &row.to_string();
        bar.truncate(n_cols - min(row.len(), n_cols));
        while n_cols - bar.len() > row.len() {
            write!(bar, " ")?;
        }
        write!(bar, "{}", row)?;
        write!(s, "{}", bar.negative())?;
        Ok(())
    }

    fn recalculate_cursor_pos(&mut self) {
        let row_pos = self.y_cursor_pos as usize + self.row_offset; //position in the file
        if row_pos < self.render_buffer.len() {
            if self.x_cursor_pos as usize + self.col_offset > self.render_buffer[row_pos].len() {
                self.x_cursor_pos =
                    self.render_buffer[self.y_cursor_pos as usize + self.row_offset].len() as u16;
            }
        } else {
            if self.x_cursor_pos > 0 {
                self.x_cursor_pos = 0;
            }
        }
    }

    fn move_cursor(&mut self, key: KeyEvent) -> Result<()> {
        let (n_cols, n_rows) = term_size()?;
        let n_rows = n_rows - 1; //Space for the status bar
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

    fn execute_command(&mut self) -> Result<()> {
        match self.command_buffer.as_ref() {
            ":q" => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_bar_not_panic_with_litte_windows() -> Result<()> {
        let editor = Editor::new();
        let mut s = String::new();
        editor.draw_status_bar(&mut s, 0)
    }
}
