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
    enableRawMode();

    let mut c = [0; 1];
    loop {
        io::stdin().read_exact(&mut c)?;
        if c[0] == 'q' as u8 {
            break Ok(());
        }
    }
}

fn enableRawMode() {
    let mut raw: MaybeUninit<termios> = MaybeUninit::uninit();
    unsafe {
        tcgetattr(STDIN_FILENO as i32, raw.as_mut_ptr());
        let mut config = raw.assume_init();
        ORIGINAL_CONFIG = Some(config.clone());
        config.c_lflag &= !(ECHO);
        tcsetattr(STDIN_FILENO as i32, TCSAFLUSH as i32, &mut config);
        atexit(Some(disableRawMode));
    }
}

extern "C" fn disableRawMode() {
    unsafe {
        tcsetattr(
            STDIN_FILENO as i32,
            TCSAFLUSH as i32,
            &mut ORIGINAL_CONFIG.unwrap(),
        );
    }
}
