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
pub struct TerminalInterface {
    pub tx: Sender<InterfaceMessage>
}

impl TerminalInterface {
    pub fn start (signature: clock::ClockSignature, control_tx: Sender<control::ControlMessage>) -> Self {
        let (tx, rx) = channel();

        let interface = Self {
            tx
        };

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
                        control_tx.send(control::ControlMessage::TapTempo).unwrap();
                    }

                    // https://github.com/jeaye/ncurses-rs/blob/master/src/constants.rs
                    Some(WchResult::KeyCode(_)) => {}

                    // Some(WchResult::KeyCode(KEY_ENTER)) => beat(),
                    Some(WchResult::Char(ch)) => {
                        if (ch == CHAR_SPACE || ch == CHAR_NEWLINE) {
                            control_tx.send(control::ControlMessage::TapTempo).unwrap();
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
                        print_time(time);
                        print_signature(signature);
                    }
                    InterfaceMessage::Signature(signature) => {
                    }
                }

                ncurses::refresh();
            }
        });

        interface
    }
}

pub fn print_time (time: clock::ClockTime) {
    ncurses::printw("nanos: ");
    ncurses::printw(format!("{}\n", time.nanos).as_ref());
    ncurses::printw("ticks: ");
    ncurses::printw(format!("{}\n", time.ticks).as_ref());
    ncurses::printw("beats: ");
    ncurses::printw(format!("{}\n", time.beats).as_ref());
    ncurses::printw("bars: ");
    ncurses::printw(format!("{}\n", time.bars).as_ref());
}

pub fn print_signature (signature: clock::ClockSignature) {
    ncurses::printw("beats per minute: ");
    ncurses::printw(format!("{}\n", signature.to_beats_per_minute()).as_ref());
    ncurses::printw("ticks per beat: ");
    ncurses::printw(format!("{}\n", signature.ticks_per_beat).as_ref());
    ncurses::printw("beats per bar: ");
    ncurses::printw(format!("{}\n", signature.beats_per_bar).as_ref());
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum InterfaceMessage {
    Time(clock::ClockTime),
    Signature(clock::ClockSignature),
}
