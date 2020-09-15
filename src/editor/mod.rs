use super::{line::Line, modes::Mode};
use crossterm::{
    execute,
    terminal::{enable_raw_mode, EnterAlternateScreen},
    Result,
};
use std::env;
use std::io;
use std::io::prelude::*;
mod cursor;
use cursor::Cursor;
mod command;
mod event;
mod file;
mod insert;
mod normal;
mod screen;

pub struct Editor {
    buffer: Vec<Line>,
    cursor: Cursor,
    y_scroll: usize,
    file_name: String,
    mode: Mode,
    command_buffer: String,
}

impl Editor {
    pub fn init() -> Result<()> {
        let mut editor = Editor::new();
        let args: Vec<String> = env::args().collect();
        if args.len() >= 2 {
            editor.load_to_buf(&args[1]).expect(
                "Invalid path or file. Keep in mind that tinate can only read Unicode valid files",
            );
        } else {
            println!("You must call tinate with the name of the file you want to read or create");
            return Ok(());
        }
        execute!(io::stdout(), EnterAlternateScreen)?;
        enable_raw_mode()?;
        loop {
            editor.refresh_screen()?;
            editor.process_event()?;
        }
    }

    pub fn new() -> Self {
        Editor {
            buffer: Vec::new(),
            cursor: Cursor::new(),
            y_scroll: 0,
            file_name: String::new(),
            mode: Mode::Normal,
            command_buffer: String::new(),
        }
    }
}
