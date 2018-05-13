extern crate ncurses;

use ncurses::{WchResult};
use std::char;
use std::sync::mpsc::{channel, Sender, Receiver};
use std::thread::{sleep, spawn};
use std::time::{Duration};

mod clock;
mod control;

// https://unicode.org/charts/PDF/U0000.pdf
static CHAR_SPACE: u32 = 0x0020;
static CHAR_RETURN: u32 = 0x000D;
static CHAR_NEWLINE: u32 = 0x000A;

fn main () {
    let control = control::Control::new();

    let clock_signature = clock::ClockSignature::new(60_f64);
    let mut clock = clock::Clock::new(clock_signature, &control);

    clock.start();

    let terminal_interface = TerminalInterface::new(&control);
    
    terminal_interface.start();

    for control_message in control.rx {
        match control_message {
            control::ControlMessage::Time(time) => {
                terminal_interface.tx.send(InterfaceMessage::Time(time));
            }
        }
    }
}

#[derive(Debug)]
pub struct TerminalInterface {
    control_tx: Sender<control::ControlMessage>,
    tx: Sender<InterfaceMessage>,
    rx: Receiver<InterfaceMessage>
}

impl TerminalInterface {
    pub fn new (control: &control::Control) -> Self {
        let (tx, rx) = channel();
        
        Self {
            control_tx: control.tx.clone(),
            tx,
            rx
        }
    }

    pub fn start (&self) {
        let control_tx = self.control_tx.clone();

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
            for interface_message in self.rx {
                match interface_message {
                    InterfaceMessage::Time(time) => {
                        print_time(time);
                    }
                }

            }
        });
    }
}

pub fn print_time (time: clock::ClockTime) {
    ncurses::clear();
    ncurses::mv(0, 0);
    ncurses::printw("nanos: ");
    ncurses::printw(format!("{}\n", time.nanos).as_ref());
    ncurses::printw("\nticks: ");
    ncurses::printw(format!("{}\n", time.ticks).as_ref());
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum InterfaceMessage {
    Time(clock::ClockTime)
}
