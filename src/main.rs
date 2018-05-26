extern crate ncurses;
extern crate num;

mod metronome;
mod clock;
mod interface;

fn main () {
    let control = metronome::Metronome::new();
    control.run();
}
