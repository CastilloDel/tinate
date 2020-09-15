use super::{Editor, Mode};
use crossterm::{
    event::{Event, KeyCode, KeyEvent},
    Result,
};

impl Editor {
    pub(super) fn match_event_normal(&mut self, event: Event) -> Result<()> {
        match event {
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
            Event::Key(KeyEvent {
                code: KeyCode::Char('A'),
                ..
            }) => {
                self.move_cursor_right(self.buffer[self.y()].len(), false);
                self.mode = Mode::Insert;
            }
            _ => {}
        }
        Ok(())
    }
}
