#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use std::io;
use std::io::prelude::*;
use std::process;
use termios::{
    os::target::TCSAFLUSH, tcsetattr, Termios, ECHO, ICANON, ICRNL, IEXTEN, ISIG, IXON, OPOST,
};

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

static mut ORIGINAL_CONFIG: Option<Termios> = None;

fn main() -> Result<(), io::Error> {
    prepareTermConfig();

    loop {
        refreshScreen();
        processKey();
    }
}

fn processKey() {
    let key = readKey();

    match key {
        key if key == controlKey('q' as u8) => {
            clearScreen();
            process::exit(0);
        }
        _ => {}
    }
}

fn readKey() -> u8 {
    let mut key = [0; 1];
    match io::stdin().read_exact(&mut key) {
        Ok(()) => key[0],
        Err(error) => panic!("Couldn't read: {}", error),
    }
}

fn refreshScreen() {
    clearScreen();

    drawRows();

    print!("\x1b[H");
    io::stdout().flush().expect("Couldn't flush the stdout");
}

fn clearScreen() {
    print!("\x1b[2J");
    print!("\x1b[H");
    io::stdout().flush().expect("Couldn't flush the stdout");
}

fn drawRows() {
    for _i in 0..24 {
        println!("~\r");
    }
}

///Enables raw mode and disables canonical mode as well as other things
fn prepareTermConfig() {
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
    unsafe {
        atexit(Some(restoreTermConfig));
    }
}

extern "C" fn restoreTermConfig() {
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

fn controlKey(key: u8) -> u8 {
    key & 0x1f
}

fn die(msg: &'static str) -> ! {
    clearScreen();
    panic!(msg);
}
