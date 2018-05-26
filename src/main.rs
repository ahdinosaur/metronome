extern crate ncurses;
extern crate num;

use std::sync::mpsc::{channel, Sender, Receiver};

mod metronome;
mod clock;
mod interface;

fn main () {
    let control = metronome::Metronome::new();
    control.run();
}
