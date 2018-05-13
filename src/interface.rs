extern crate ncurses;

use ncurses::{WchResult};
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::{sleep, spawn};

use clock;
use control;

// https://unicode.org/charts/PDF/U0000.pdf
static CHAR_SPACE: u32 = 0x0020;
static CHAR_RETURN: u32 = 0x000D;
static CHAR_NEWLINE: u32 = 0x000A;

#[derive(Debug)]
pub struct TerminalInterface {}

impl TerminalInterface {
    pub fn start (control_tx: Sender<control::ControlMessage>) -> Sender<InterfaceMessage> {
        let (tx, rx) = channel();

        let mut signature = clock::ClockSignature::default();

        spawn(move|| {
            /* Setup ncurses. */
            ncurses::initscr();

            let locale_conf = ncurses::LcCategory::all;
            ncurses::setlocale(locale_conf, "en_US.UTF-8");

            /* Enable mouse events. */
            ncurses::mousemask(ncurses::ALL_MOUSE_EVENTS as ncurses::mmask_t, None);

            /* Allow for extended keyboard (like F1). */
            ncurses::keypad(ncurses::stdscr(), true);
            ncurses::noecho();

            loop {
                let ch = ncurses::wget_wch(ncurses::stdscr());

                match ch {
                    Some(WchResult::KeyCode(ncurses::KEY_MOUSE)) => {
                    }

                    Some(WchResult::KeyCode(ncurses::KEY_UP)) => {
                        control_tx.send(control::ControlMessage::NudgeTempo(1_f64)).unwrap();
                    }

                    Some(WchResult::KeyCode(ncurses::KEY_DOWN)) => {
                        control_tx.send(control::ControlMessage::NudgeTempo(-1_f64)).unwrap();
                    }

                    // https://github.com/jeaye/ncurses-rs/blob/master/src/constants.rs
                    Some(WchResult::KeyCode(_)) => {
                    }

                    // Some(WchResult::KeyCode(KEY_ENTER)) => beat(),
                    Some(WchResult::Char(ch)) => {
                        if (ch == CHAR_SPACE) {
                            control_tx.send(control::ControlMessage::Tap).unwrap();
                        }

                        if (ch == CHAR_NEWLINE) {
                            control_tx.send(control::ControlMessage::Reset).unwrap();
                        }
                    }

                    None => {}
                }

                ncurses::refresh();
            }

            ncurses::endwin();
        });

        spawn(move|| {
            for interface_message in rx {
                match interface_message {
                    InterfaceMessage::Time(time) => {
                        ncurses::clear();
                        ncurses::mv(0, 0);
                        print_beat(time);
                        print_bar(time);
                        print_time(time);
                        print_signature(signature);
                    },
                    InterfaceMessage::Signature(next_signature) => {
                        signature = next_signature;
                    }
                }

                ncurses::refresh();
            }
        });

        tx
    }
}

pub fn print_beat (time: clock::ClockTime) {
    if time.ticks() == 0 {
        if time.beats() == 0 {
            ncurses::printw("SUPER ");
        }
        ncurses::printw("BEAT");
    }
    ncurses::printw("\n");
}

pub fn print_bar (time: clock::ClockTime) {
    if time.bars() == 0 {
        ncurses::printw("YAY YAY YAY");
    }
    ncurses::printw("\n");
}

pub fn print_time (time: clock::ClockTime) {
    ncurses::printw("nanos: ");
    ncurses::printw(format!("{}\n", time.nanos()).as_ref());
    ncurses::printw("ticks: ");
    ncurses::printw(format!("{}\n", time.ticks() + 1).as_ref());
    ncurses::printw("beats: ");
    ncurses::printw(format!("{}\n", time.beats() + 1).as_ref());
    ncurses::printw("bars: ");
    ncurses::printw(format!("{}\n", time.bars() + 1).as_ref());
}

pub fn print_signature (signature: clock::ClockSignature) {
    ncurses::printw("beats per minute: ");
    ncurses::printw(format!("{}\n", signature.to_beats_per_minute()).as_ref());
    ncurses::printw("ticks per beat: ");
    ncurses::printw(format!("{}\n", signature.ticks_per_beat).as_ref());
    ncurses::printw("beats per bar: ");
    ncurses::printw(format!("{}\n", signature.beats_per_bar).as_ref());
    ncurses::printw("bars per loop: ");
    ncurses::printw(format!("{}\n", signature.bars_per_loop).as_ref());
}

#[derive(Clone, Copy, Debug)]
pub enum InterfaceMessage {
    Time(clock::ClockTime),
    Signature(clock::ClockSignature),
}
