use std::io;
use std::io::prelude::*;
use termios::{
    os::target::TCSAFLUSH, tcsetattr, Termios, ECHO, ICANON, ICRNL, IEXTEN, ISIG, IXON, OPOST,
};

const STDIN_FILENO: u32 = 0;

fn main() -> Result<(), io::Error> {
    let original_config = get_term_config();
    prepare_term_config();
    let _will_get_dropped = AtExit { original_config };

    loop {
        refresh_screen();
        if let None = process_key() {
            break Ok(());
        }
    }
}

struct AtExit {
    original_config: Termios,
}

impl Drop for AtExit {
    fn drop(&mut self) {
        set_term_config(&mut self.original_config);
    }
}

fn process_key() -> Option<()> {
    let key = read_key();

    match key {
        key if key == control_key('q' as u8) => {
            clear_screen();
            None
        }
        _ => Some(()),
    }
}

fn read_key() -> u8 {
    let mut key = [0; 1];
    match io::stdin().read_exact(&mut key) {
        Ok(()) => key[0],
        Err(error) => panic!("Couldn't read: {}", error),
    }
}

fn refresh_screen() {
    clear_screen();

    draw_rows();

    print!("\x1b[H");
    io::stdout().flush().expect("Couldn't flush the stdout");
}

fn clear_screen() {
    print!("\x1b[2J");
    print!("\x1b[H");
    io::stdout().flush().expect("Couldn't flush the stdout");
}

fn draw_rows() {
    for _i in 0..24 {
        println!("~\r");
    }
}

fn get_term_config() -> Termios {
    match Termios::from_fd(STDIN_FILENO as i32) {
        Err(_) => die("Couldn't get the terminal configuration"),
        Ok(config) => config,
    }
}

///Enables raw mode and disables canonical mode as well as other things
fn prepare_term_config() {
    let mut config = get_term_config();
    config.c_iflag &= !(IXON | ICRNL);
    config.c_oflag &= !(OPOST);
    config.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    set_term_config(&mut config);
}

fn set_term_config(config: &mut Termios) {
    if let Err(_) = tcsetattr(STDIN_FILENO as i32, TCSAFLUSH as i32, config) {
        die("Couldn't set the terminal configuration");
    }
}

fn control_key(key: u8) -> u8 {
    key & 0x1f
}

fn die(msg: &'static str) -> ! {
    clear_screen();
    panic!(msg);
}
