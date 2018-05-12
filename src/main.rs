use std::io;
use std::io::prelude::*;

use std::thread::sleep;
use std::time::{Duration};

fn main () {
    metronome().unwrap();
}

fn metronome () -> io::Result<()> {
    let stdin = io::stdin();
    let mut buffer = vec![0_u8; 2_usize.pow(8)];

    loop {
        let mut handle = stdin.lock();

        match handle.read(&mut buffer) {
            Ok(num_bytes) => {
                println!("{} bytes read", num_bytes);
                let string = String::from_utf8(buffer.clone()).unwrap();
            },
            Err(error) => println!("error: {}", error),
        }

        sleep(Duration::new(0, 30 * 1000));
    }
}
