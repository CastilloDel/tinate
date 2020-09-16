use super::{Editor, Mode};
use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, LeaveAlternateScreen},
    Result,
};
use std::io;
use std::io::prelude::*;
use std::process::exit;

impl Editor {
    pub(super) fn match_event_command(&mut self, event: Event) -> Result<()> {
        match event {
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
            Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            }) => {
                self.command_buffer.pop();
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
            ":w" => {
                self.mode = Mode::Normal;
                self.save_to_file()
            }
            ":wq" => {
                self.save_to_file()?;
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
}
