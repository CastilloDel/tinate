use super::{Editor, Mode};
use crossterm::{event::read, Result};

impl Editor {
    pub(super) fn process_event(&mut self) -> Result<()> {
        let event = read()?;

        match self.mode {
            Mode::Normal => self.match_event_normal(event),
            Mode::Command => self.match_event_command(event),
            Mode::Insert => self.match_event_insert(event),
        }
    }
}
