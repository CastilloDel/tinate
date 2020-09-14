use crossterm::Result;
mod editor;
use editor::Editor;
mod line;
mod modes;

fn main() -> Result<()> {
    Editor::init()
}
