use std::io;
use std::io::prelude::*;
use std::process;
use termios::{
    os::target::TCSAFLUSH, tcsetattr, Termios, ECHO, ICANON, ICRNL, IEXTEN, ISIG, IXON, OPOST,
};

static mut ORIGINAL_CONFIG: Option<Termios> = None;
const STDIN_FILENO: u32 = 0;

fn main() -> Result<(), io::Error> {
    prepare_term_config();
    let _will_get_dropped = AtExit {};

    loop {
        refresh_screen();
        process_key();
    }
}

struct AtExit;

impl Drop for AtExit {
    fn drop(&mut self) {
        restore_term_config();
    }
}

fn process_key() {
    let key = read_key();

    match key {
        key if key == control_key('q' as u8) => {
            clear_screen();
            process::exit(0);
        }
        _ => {}
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

///Enables raw mode and disables canonical mode as well as other things
fn prepare_term_config() {
    let mut config = match Termios::from_fd(STDIN_FILENO as i32) {
        Err(_) => die("Couldn't get the terminal configuration"),
        Ok(config) => config,
    };
    //SAFETY: Accesing a global static variable
    unsafe {
        if let None = ORIGINAL_CONFIG {
            ORIGINAL_CONFIG = Some(config.clone());
        }
    }
    config.c_iflag &= !(IXON | ICRNL);
    config.c_oflag &= !(OPOST);
    config.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    if let Err(_) = tcsetattr(STDIN_FILENO as i32, TCSAFLUSH as i32, &mut config) {
        die("Couldn't set the terminal configuration");
    }
}

fn restore_term_config() {
    //SAFETY: Accesing a global static variable
    unsafe {
        if let Err(_) = tcsetattr(
            STDIN_FILENO as i32,
            TCSAFLUSH as i32,
            &mut ORIGINAL_CONFIG.unwrap(),
        ) {
            die("Couldn't restore the original terminal configuration");
        }
    }
}

fn control_key(key: u8) -> u8 {
    key & 0x1f
}

fn die(msg: &'static str) -> ! {
    clear_screen();
    panic!(msg);
}
