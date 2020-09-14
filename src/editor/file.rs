use super::{Editor, Line};
use crossterm::Result;
use std::fs::File;
use std::io;
use std::io::prelude::*;

impl Editor {
    pub(super) fn load_to_buf(&mut self, path: &str) -> io::Result<()> {
        self.file_name = path.to_owned();
        match File::open(&self.file_name) {
            Ok(file) => {
                self.buffer = io::BufReader::new(file)
                    .lines()
                    .map(|line_result| line_result.map(|line| Line::new(&line)))
                    .collect::<io::Result<Vec<Line>>>()?;
            }
            Err(err) if err.kind() == io::ErrorKind::NotFound => {}
            Err(err) => return Err(err),
        }
        Ok(())
    }

    pub(super) fn save_to_file(&self) -> Result<()> {
        let mut file = File::create(&self.file_name)?;
        for line in self.buffer.iter() {
            file.write(line.get_content().as_bytes())?;
            file.write("\n".as_bytes())?;
        }
        Ok(())
    }
}
