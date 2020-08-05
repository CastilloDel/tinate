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
            println!("{}", c[0]);
        } else {
            println!("{} ({})", c[0], c[0] as char);
        }
    }
}

///Enables raw mode and disables canonical mode
fn prepareTermConfig() {
    let mut raw: MaybeUninit<termios> = MaybeUninit::uninit();
    let mut config;
    unsafe {
        tcgetattr(STDIN_FILENO as i32, raw.as_mut_ptr());
        config = raw.assume_init();
        if let None = ORIGINAL_CONFIG {
            ORIGINAL_CONFIG = Some(config.clone());
        }
        config.c_lflag &= !(ECHO | ICANON);
        tcsetattr(STDIN_FILENO as i32, TCSAFLUSH as i32, &mut config);
        atexit(Some(restoreTermConfig));
    }
}

extern "C" fn restoreTermConfig() {
    unsafe {
        tcsetattr(
            STDIN_FILENO as i32,
            TCSAFLUSH as i32,
            &mut ORIGINAL_CONFIG.unwrap(),
        );
    }
}
