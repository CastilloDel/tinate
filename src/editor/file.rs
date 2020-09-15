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
            Err(err) if err.kind() == io::ErrorKind::NotFound => self.buffer = vec![Line::new("")],
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn load_from_file() -> Result<()> {
        let name = "TestFileWithANameUnnecessarilyLongToAvoidCollisions";
        let mut file = File::create(name)?;
        file.write(b"This is a line\nAnd this is another line")?;
        let mut editor = Editor::new();
        editor.load_to_buf(name)?;
        assert_eq!(
            editor.buffer,
            vec![
                Line::new("This is a line"),
                Line::new("And this is another line")
            ]
        );
        std::fs::remove_file(name)?;
        Ok(())
    }

    #[test]
    fn save() -> Result<()> {
        let mut editor = Editor::new();
        editor.buffer.push(Line::new("I will store this and an ñ"));
        let name = "TestFileWithANameUnnecessarilyLongToAvoidCollisions2";
        editor.file_name = name.to_owned();
        editor.save_to_file()?;
        let file = File::open(name)?;
        assert_eq!(
            "I will store this and an ñ",
            io::BufReader::new(file).lines().next().unwrap().unwrap()
        );
        std::fs::remove_file(name)?;
        Ok(())
    }
}
