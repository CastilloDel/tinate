#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]

use std::io;
use std::io::Read;
use std::mem::MaybeUninit;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

static mut ORIGINAL_CONFIG: Option<termios> = None;

fn main() -> Result<(), io::Error> {
    prepareTermConfig();

    let mut c = [0; 1];
    loop {
        io::stdin().read_exact(&mut c)?;
        if c[0] == 'q' as u8 {
            break Ok(());
        }
        if c[0].is_ascii_control() {
            println!("{}\r", c[0]);
        } else {
            println!("{} ({})\r", c[0], c[0] as char);
        }
    }
}

///Enables raw mode and disables canonical mode
fn prepareTermConfig() {
    let mut raw: MaybeUninit<termios> = MaybeUninit::uninit();
    let mut config;
    let mut result;
    unsafe {
        result = tcgetattr(STDIN_FILENO as i32, raw.as_mut_ptr());
    }
    if result == -1 {
        panic!("Couldn't get the terminal configuration");
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
        panic!("Couldn't set the terminal configuration");
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
        panic!("Couldn't restore the original terminal configuration")
    }
}
