#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use std::io;
use std::io::prelude::*;
use std::mem::MaybeUninit;
use std::process;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

static mut ORIGINAL_CONFIG: Option<termios> = None;

fn main() -> Result<(), io::Error> {
    prepareTermConfig();

    loop {
        clearScreen();
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

fn clearScreen() {
    print!("\x1b[2J");
    print!("\x1b[H");
    io::stdout().flush().expect("Couldn't flush the stdout");
}

///Enables raw mode and disables canonical mode as well as other things
fn prepareTermConfig() {
    let mut raw: MaybeUninit<termios> = MaybeUninit::uninit();
    let mut config;
    let mut result;
    unsafe {
        result = tcgetattr(STDIN_FILENO as i32, raw.as_mut_ptr());
    }
    if result == -1 {
        die("Couldn't get the terminal configuration");
    }
    unsafe {
        config = raw.assume_init();
        if let None = ORIGINAL_CONFIG {
            ORIGINAL_CONFIG = Some(config.clone());
        }
    }
    config.c_iflag &= !(IXON | IXON);
    config.c_oflag &= !(OPOST);
    config.c_lflag &= !(ECHO | ICANON | IEXTEN | ISIG);
    unsafe {
        result = tcsetattr(STDIN_FILENO as i32, TCSAFLUSH as i32, &mut config);
        atexit(Some(restoreTermConfig));
    }
    if result == -1 {
        die("Couldn't set the terminal configuration");
    }
}

extern "C" fn restoreTermConfig() {
    let result;
    unsafe {
        result = tcsetattr(
            STDIN_FILENO as i32,
            TCSAFLUSH as i32,
            &mut ORIGINAL_CONFIG.unwrap(),
        );
    }
    if result == -1 {
        die("Couldn't restore the original terminal configuration");
    }
}

fn controlKey(key: u8) -> u8 {
    key & 0x1f
}

fn die(msg: &'static str) {
    clearScreen();
    panic!(msg);
}
