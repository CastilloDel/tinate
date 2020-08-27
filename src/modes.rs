use std::fmt::{Display, Formatter, Result};

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    Normal,
    Insert,
    Command,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Mode::Normal => write!(f, "Normal"),
            Mode::Insert => write!(f, "Insert"),
            Mode::Command => write!(f, "Command"),
        }
    }
}
