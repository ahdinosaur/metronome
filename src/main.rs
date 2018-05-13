/*
use std::io;
use std::io::prelude::*;

use std::sync::mpsc;
use std::thread::sleep;
use std::time::{Duration};

fn main () {
    let metronome = Metronone::new();
    metronome().unwrap();
}

pub struct Metronone {}

impl Metronome () {
    pub fn new () -> <()> {
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

            sleep(Duration::new(0, 30 * 1000 * 1000));
        }
    }
*/

/*
    Copyright © 2013 Free Software Foundation, Inc
    See licensing in LICENSE file
    File: examples/ex_7.rs
    Author: Jesse 'Jeaye' Wilkerson
    Description:
      Basic input and attribute example, using the Unicode-aware get_wch functions.
*/

extern crate ncurses;
// extern crate ctrlc;

use std::char;
use ncurses::{WchResult};
use std::thread::{sleep,spawn};
use std::time::{Duration};

mod clock;

// https://unicode.org/charts/PDF/U0000.pdf
static CHAR_SPACE: u32 = 0x0020;
static CHAR_RETURN: u32 = 0x000D;
static CHAR_NEWLINE: u32 = 0x000A;

fn main () {
    clock();
    // terminal_interface();
    
    loop {
        sleep(Duration::new(10, 0));
    }
}

fn clock () {
    spawn(move|| {
        let time_signature = clock::TimeSignature {
            nanos_per_beat: BEATS_PER_MINUTE / SECONDS_PER_MINUTE * NANOS_PER_SECOND,
            ticks_per_beat: DEFAULT_TICKS_PER_BEAT,
            beats_per_measure: DEFAULT_BEATS_PER_MEASURE
        };
        let mut clock = clock::Clock::new(time_signature);
        loop {
            clock.tick();
            println!("{:?}", clock.get_time());
        }
    });
}

fn terminal_interface () {
    spawn(move|| {
        let locale_conf = ncurses::LcCategory::all;
        ncurses::setlocale(locale_conf, "en_US.UTF-8");

        /* Setup ncurses. */
        ncurses::initscr();

        /* Enable mouse events. */
        ncurses::mousemask(ncurses::ALL_MOUSE_EVENTS as ncurses::mmask_t, None);

        /* Allow for extended keyboard (like F1). */
        ncurses::keypad(ncurses::stdscr(), true);
        ncurses::noecho();

        loop {
            let ch = ncurses::wget_wch(ncurses::stdscr());

            match ch {
                Some(WchResult::KeyCode(ncurses::KEY_MOUSE)) => {
                    tap();
                }

                // https://github.com/jeaye/ncurses-rs/blob/master/src/constants.rs
                Some(WchResult::KeyCode(_)) => {}

                // Some(WchResult::KeyCode(KEY_ENTER)) => beat(),
                Some(WchResult::Char(ch)) => {
                    if (ch == CHAR_SPACE || ch == CHAR_NEWLINE) {
                        tap();
                    }
                }

                None => {}
            }

            ncurses::refresh();
        }

        ncurses::endwin();
    });
}

fn tap () {
    ncurses::attron(ncurses::A_BOLD());
    ncurses::printw("\nBeat");
    ncurses::attroff(ncurses::A_BOLD());
}
