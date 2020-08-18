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
use std::cmp::min;
use std::env;
use std::fmt::Write as fmt_write;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::process::exit;
use std::sync::atomic::{AtomicU16, AtomicUsize, Ordering::Relaxed};

const WELCOME_MESSAGE: &'static str = "Tinate Is Not A Text Editor";
static X: AtomicU16 = AtomicU16::new(0);
static Y: AtomicU16 = AtomicU16::new(0);
static ROW_OFFSET: AtomicUsize = AtomicUsize::new(0);
static COL_OFFSET: AtomicUsize = AtomicUsize::new(0);

fn main() -> Result<()> {
    execute!(io::stdout(), EnterAlternateScreen)?;
    enable_raw_mode()?;
    let _will_get_dropped = AtExit {};

    let args: Vec<String> = env::args().collect();
    let mut buffer: Vec<String> = Vec::new();
    if args.len() >= 2 {
        buffer = load_to_buf(&args[1]).expect("Invalid path or file");
    }

    loop {
        refresh_screen(&buffer)?;
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

fn load_to_buf(path: &str) -> io::Result<Vec<String>> {
    let file = File::open(path)?;
    let buffer: Vec<String> = io::BufReader::new(file)
        .lines()
        .map(|line_result| line_result.map(|line| line.trim_end().to_string()))
        .collect::<io::Result<Vec<String>>>()?;
    Ok(buffer)
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
        key if is_movement_key(key) => move_cursor(key),
        _ => Ok(()),
    }
}

fn read_key() -> Result<u8> {
    let mut key = [0; 1];
    io::stdin().read_exact(&mut key)?;
    Ok(key[0])
}

fn refresh_screen(buf: &Vec<String>) -> Result<()> {
    draw_rows(buf)?;

    io::stdout().flush()?;
    Ok(())
}

fn draw_rows(buf: &Vec<String>) -> Result<()> {
    let mut s = String::new();
    let (n_cols, n_rows) = term_size()?;
    let (n_cols, n_rows) = (n_cols as usize, n_rows as usize);
    queue!(s, MoveTo(0, 0))?;
    let row_offset = ROW_OFFSET.load(Relaxed);
    let col_offset = COL_OFFSET.load(Relaxed);
    for i in 0..n_rows {
        queue!(s, Clear(ClearType::CurrentLine))?;
        if i < buf.len() - min(row_offset, buf.len()) {
            let trunc_line = trunc_line(&buf[i + row_offset], n_cols, col_offset);
            if i == n_rows - 1 {
                write!(&mut s, "{}\r", &trunc_line)?;
            } else {
                write!(&mut s, "{}\r\n", &trunc_line)?;
            }
        } else {
            if buf.len() == 0 && i == n_rows / 3 {
                add_welcome_message(&mut s, n_cols)?;
            } else if i == n_rows - 1 {
                write!(&mut s, "~")?;
            } else {
                write!(&mut s, "~\r\n")?;
            }
        }
    }
    if buf.len() != 0 {
        recalculate_cursor_pos(buf);
    }
    queue!(s, MoveTo(X.load(Relaxed), Y.load(Relaxed)))?;
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
        write_padding(s, n_cols)?;
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

fn recalculate_cursor_pos(buf: &Vec<String>) {
    let x = X.load(Relaxed) as usize;
    let y = Y.load(Relaxed) as usize;
    let row_offset = ROW_OFFSET.load(Relaxed);
    let row_pos = y + row_offset; //position in the file
    if row_pos < buf.len() {
        if x + COL_OFFSET.load(Relaxed) > buf[row_pos].len() {
            X.store(buf[y + row_offset].len() as u16, Relaxed);
        }
    } else {
        if x > 0 {
            X.store(0, Relaxed);
        }
    }
}

fn move_cursor(key: u8) -> Result<()> {
    let (n_cols, n_rows) = term_size()?;
    let x = X.load(Relaxed);
    let y = Y.load(Relaxed);
    match key {
        key if key == 'h' as u8 => {
            if x > 0 {
                X.store(x - 1, Relaxed);
            } else {
                if COL_OFFSET.load(Relaxed) > 0 {
                    COL_OFFSET.fetch_sub(1, Relaxed);
                }
                if x > 0 {
                    X.store(0, Relaxed);
                }
            }
        }
        key if key == 'j' as u8 => {
            if y < (n_rows - 1) {
                Y.store(y + 1, Relaxed);
            } else {
                ROW_OFFSET.fetch_add(1, Relaxed);
                Y.store(n_rows - 1, Relaxed);
            }
        }
        key if key == 'k' as u8 => {
            if y > 0 {
                Y.store(y - 1, Relaxed);
            } else {
                if ROW_OFFSET.load(Relaxed) > 0 {
                    ROW_OFFSET.fetch_sub(1, Relaxed);
                }
                if y > 0 {
                    Y.store(0, Relaxed);
                }
            }
        }
        key if key == 'l' as u8 => {
            if x < (n_cols - 1) {
                X.store(x + 1, Relaxed);
            } else {
                COL_OFFSET.fetch_add(1, Relaxed);
                X.store(n_cols - 1, Relaxed);
            }
        }
        _ => {}
    }
    Ok(())
}

fn control_key(key: u8) -> u8 {
    key & 0x1f
}

fn is_movement_key(key: u8) -> bool {
    key == 'h' as u8 || key == 'j' as u8 || key == 'k' as u8 || key == 'l' as u8
}
