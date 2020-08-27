use std::fmt::{Display, Formatter, Result};

pub enum Mode {
    Normal,
    Insert,
}

impl Display for Mode {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Mode::Normal => write!(f, "Normal"),
            Mode::Insert => write!(f, "Insert"),
        }
    }
}
