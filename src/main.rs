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
use std::cmp::{max, min};
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
        execute!(io::stdout(), LeaveAlternateScreen).ok();
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
        match File::open(&self.file_name) {
            Ok(file) => {
                self.buffer = io::BufReader::new(file)
                    .lines()
                    .map(|line_result| line_result.map(|line| line.trim_end().to_string()))
                    .collect::<io::Result<Vec<String>>>()?;
                self.update_render_buf();
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
        Ok(())
    }

    fn update_render_buf(&mut self) {
        for index in 0..self.buffer.len() {
            self.render_buffer.push(String::new());
            self.update_render_row(index);
        }
    }

    fn update_render_row(&mut self, index: usize) {
        self.render_buffer[index].clear();
        for c in self.buffer[index].chars() {
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
                Event::Key(key) if Editor::is_movement_key(&key) => {
                    self.move_cursor_safely(key.code)
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('i'),
                    ..
                }) => {
                    self.mode = Mode::Insert;
                    Ok(())
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Char('a'),
                    ..
                }) => {
                    let row_pos = self.y_cursor_pos as usize + self.row_offset; //position in the file
                    if row_pos < self.buffer.len() {
                        self.x_cursor_pos += 1;
                    }
                    self.mode = Mode::Insert;
                    Ok(())
                }
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
                Event::Key(KeyEvent {
                    code: KeyCode::Char(c),
                    ..
                }) => {
                    self.insert_char(c);
                    Ok(())
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                }) => {
                    self.move_cursor_safely(KeyCode::Char('h'))?;
                    self.mode = Mode::Normal;
                    Ok(())
                }
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
                if self.file_name == "" && i == n_rows / 3 {
                    Editor::add_welcome_message(&mut s, n_cols)?;
                }
                write!(&mut s, "~\r\n")?;
            }
        }
        self.draw_status_bar(&mut s, n_cols)?;
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

    fn move_cursor_safely(&mut self, key: KeyCode) -> Result<()> {
        self.move_cursor(key)?;
        self.recalculate_cursor_pos();
        self.avoid_tabs(key)?;
        Ok(())
    }

    fn move_cursor(&mut self, key: KeyCode) -> Result<()> {
        let (n_cols, n_rows) = term_size()?;
        let n_rows = n_rows - 1; //Space for the status bar
        match key {
            KeyCode::Char('h') => {
                if self.x_cursor_pos > 0 {
                    self.x_cursor_pos -= 1;
                } else {
                    if self.col_offset > 0 {
                        self.col_offset -= 1;
                    }
                    self.x_cursor_pos = 0;
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
                    self.y_cursor_pos = 0;
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

    fn recalculate_cursor_pos(&mut self) {
        let mut row_pos = self.y_cursor_pos as usize + self.row_offset; //position in the file
        if row_pos >= self.render_buffer.len() {
            self.y_cursor_pos = (max(self.render_buffer.len() - self.row_offset, 1) - 1) as u16;
            row_pos = self.y_cursor_pos as usize + self.row_offset;
            if row_pos == 0 {
                self.x_cursor_pos = 0;
                return;
            }
        }

        if self.x_cursor_pos as usize + self.col_offset >= self.render_buffer[row_pos].len() {
            self.x_cursor_pos = max(
                self.render_buffer[self.y_cursor_pos as usize + self.row_offset].len(),
                1,
            ) as u16
                - 1;
        }
    }

    fn avoid_tabs(&mut self, key: KeyCode) -> Result<()> {
        let row_pos = self.y_cursor_pos as usize + self.row_offset; //position in the file
        if self.buffer.is_empty() || self.buffer[row_pos].is_empty() {
            return Ok(());
        }
        let mut col_pos = self.x_cursor_pos as usize + self.col_offset; //position in the file
        let buf_index = Editor::translate_rend_index_to_buf(&self.buffer[row_pos], col_pos);
        if self.buffer[row_pos].chars().skip(buf_index).next() == Some('\t') {
            match key {
                KeyCode::Char('l')
                    if Editor::translate_buf_index_to_rend(&self.buffer[row_pos], buf_index)
                        != col_pos =>
                {
                    while Editor::translate_rend_index_to_buf(&self.buffer[row_pos], col_pos)
                        != buf_index + 1
                    {
                        self.move_cursor(KeyCode::Char('l'))?;
                        col_pos = self.x_cursor_pos as usize + self.col_offset; //position in the file
                    }
                }
                _ => {
                    let rend_index =
                        Editor::translate_buf_index_to_rend(&self.buffer[row_pos], buf_index);
                    while rend_index != col_pos {
                        self.move_cursor(KeyCode::Char('h'))?;
                        col_pos = self.x_cursor_pos as usize + self.col_offset; //position in the file
                    }
                }
            }
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

    fn insert_char(&mut self, c: char) {
        let row_pos = self.y_cursor_pos as usize + self.row_offset; //position in the file
        let col_pos = self.x_cursor_pos as usize + self.col_offset; //position in the file
        if row_pos == self.buffer.len() {
            self.buffer.push(String::new());
            self.render_buffer.push(String::new());
        }
        let buf_index = Editor::translate_rend_index_to_buf(&self.buffer[row_pos], col_pos);
        self.buffer[row_pos].insert(buf_index, c);
        self.update_render_row(row_pos);
        self.x_cursor_pos += 1;
    }

    fn save_buffer(&self) -> Result<()> {
        let mut file = File::create(&self.file_name)?;
        for line in self.buffer.iter() {
            file.write(line.as_bytes())?;
            file.write("\n".as_bytes())?;
        }
        Ok(())
    }

    fn translate_rend_index_to_buf(buf_line: &str, mut r_index: usize) -> usize {
        let mut render_pos = 0;
        if buf_line.len() == 0 {
            return 0;
        }
        for (index, c) in buf_line.chars().enumerate() {
            if c == '\t' {
                r_index -= min(TAB_SZ - 1 - (render_pos % TAB_SZ), r_index);
                render_pos += TAB_SZ - (render_pos % TAB_SZ);
            } else {
                render_pos += 1;
            }
            if r_index <= index {
                return index;
            }
        }
        if r_index == buf_line.len() {
            return r_index;
        }
        panic!(
            "Couldn't translate the index({}) to a valid index in the buffer line",
            r_index
        );
    }

    fn translate_buf_index_to_rend(buf_line: &str, b_index: usize) -> usize {
        if buf_line.len() == 0 {
            return 0;
        }
        if b_index >= buf_line.len() {
            panic!(
                "The index({}) is greater than the length of the line({})",
                b_index,
                buf_line.len()
            );
        }
        let mut r_index = 0;
        for (index, c) in buf_line.chars().enumerate() {
            if b_index == index {
                return r_index;
            }
            if c == '\t' {
                r_index += TAB_SZ - (r_index % TAB_SZ);
            } else {
                r_index += 1;
            }
        }
        r_index
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

    #[test]
    fn translate_to_buf_index() {
        assert_eq!(Editor::translate_rend_index_to_buf("\ta\ttt", 0), 0);
        assert_eq!(Editor::translate_rend_index_to_buf("\ta\ttt", 2), 0);
        assert_eq!(Editor::translate_rend_index_to_buf("\ta\ttt", 3), 0);
        assert_eq!(Editor::translate_rend_index_to_buf("\ta\ttt", 4), 1);
        assert_eq!(Editor::translate_rend_index_to_buf("\ta\ttt", 7), 2);
        assert_eq!(Editor::translate_rend_index_to_buf("\ta\ttt", 8), 3);
        assert_eq!(Editor::translate_rend_index_to_buf("\ta\ttt", 9), 4);
        assert_eq!(Editor::translate_rend_index_to_buf("\ta\ttt", 10), 5);
    }

    #[test]
    #[should_panic]
    fn translate_to_buf_index_panic() {
        Editor::translate_rend_index_to_buf("\ta\ttt", 11);
    }

    #[test]
    fn translate_to_rend_index() {
        assert_eq!(Editor::translate_buf_index_to_rend("\ta\ttt", 0), 0);
        assert_eq!(Editor::translate_buf_index_to_rend("\ta\ttt", 1), 4);
        assert_eq!(Editor::translate_buf_index_to_rend("\ta\ttt", 2), 5);
        assert_eq!(Editor::translate_buf_index_to_rend("\ta\ttt", 3), 8);
        assert_eq!(Editor::translate_buf_index_to_rend("\ta\ttt", 4), 9);
    }

    #[test]
    #[should_panic]
    fn translate_to_rend_index_panic() {
        Editor::translate_buf_index_to_rend("\ta\ttt", 5);
    }
}
