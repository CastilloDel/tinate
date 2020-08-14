use crossterm::terminal::size as term_size;
use crossterm::{
    cursor::MoveTo,
    execute, queue,
    terminal::{
        disable_raw_mode, enable_raw_mode, Clear, ClearType, EnterAlternateScreen,
        LeaveAlternateScreen,
    },
    Result,
};
use std::fmt::Write as fmt_write;
use std::io;
use std::io::prelude::*;
use std::io::Write;
use std::process::exit;

const WELCOME_MESSAGE: &'static str = "Tinate Is Not A Text Editor";

fn main() -> Result<()> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let _will_get_dropped = AtExit {};

    loop {
        refresh_screen()?;
        process_key()?;
    }
}

struct AtExit {}

impl Drop for AtExit {
    fn drop(&mut self) {
        //The result isn't managed because it could cause a panic during a panic
        disable_raw_mode().ok();
    }
}

fn process_key() -> Result<()> {
    let key = read_key()?;

    match key {
        key if key == control_key('q' as u8) => {
            execute!(io::stdout(), LeaveAlternateScreen)?;
            //exit won't call destructors
            disable_raw_mode()?;
            exit(0);
        }
        _ => Ok(()),
    }
}

fn read_key() -> Result<u8> {
    let mut key = [0; 1];
    io::stdin().read_exact(&mut key)?;
    Ok(key[0])
}

fn refresh_screen() -> Result<()> {
    queue!(io::stdout(), MoveTo(0, 0))?;

    draw_rows()?;

    queue!(io::stdout(), MoveTo(0, 0))?;
    io::stdout().flush()?;
    Ok(())
}

fn draw_rows() -> Result<()> {
    let mut s = String::new();
    let (n_cols, n_rows) = term_size()?;
    for i in 0..n_rows {
        queue!(io::stdout(), Clear(ClearType::CurrentLine))?;
        if i == n_rows / 3 {
            add_welcome_message(&mut s, n_cols)?;
        } else if i == n_rows - 1 {
            write!(&mut s, "~")?;
        } else {
            write!(&mut s, "~\r\n")?;
        }
    }
    print!("{}", s);
    Ok(())
}

fn add_welcome_message(s: &mut String, n_cols: u16) -> std::fmt::Result {
    let mut msg = String::from(WELCOME_MESSAGE);
    if WELCOME_MESSAGE.len() > n_cols as usize {
        msg.truncate(n_cols as usize);
    } else {
        write_padding(s, n_cols)?;
    }
    write!(s, "{}\r\n", msg)
}

fn write_padding(s: &mut String, n_cols: u16) -> std::fmt::Result {
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

fn control_key(key: u8) -> u8 {
    key & 0x1f
}
