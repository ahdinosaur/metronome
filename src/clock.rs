// inspired by https://github.com/mmckegg/rust-loop-drop/blob/master/src/midi_time.rs
// http://www.deluge.co/?q=midi-tempo-bpm

use std::u64;
use std::time::{Duration, Instant};
use std::thread::{sleep, spawn};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};

use metronome;

pub type Nanos = u64;
pub type Ticks = u64;
pub type Beats = u64;
pub type Bars = u64;

static SECONDS_PER_MINUTE: u64 = 60;
static NANOS_PER_SECOND: u64 = 1_000_000_000;
static BEATS_PER_MINUTE: u64 = 60;
static DEFAULT_TICKS_PER_BEAT: u64 = 16;
static DEFAULT_BEATS_PER_BAR: u64 = 4;
static DEFAULT_BARS_PER_LOOP: u64 = 4;
static DEFAULT_BEATS_PER_MINUTE: f64 = 60_f64;

#[derive(Clone, Copy, Debug)]
pub struct Signature {
    pub nanos_per_beat: u64, // tempo
    pub ticks_per_beat: u64, // meter
    pub beats_per_bar: u64, // meter
    pub bars_per_loop: u64,
}

impl Signature {
    pub fn new (nanos_per_beat: u64) -> Self {
        Self {
            nanos_per_beat: nanos_per_beat,
            ticks_per_beat: DEFAULT_TICKS_PER_BEAT,
            beats_per_bar: DEFAULT_BEATS_PER_BAR,
            bars_per_loop: DEFAULT_BARS_PER_LOOP,
        }
    }

    pub fn default () -> Self {
        Self::from_beats_per_minute(DEFAULT_BEATS_PER_MINUTE)
    }

    pub fn from_beats_per_minute (beats_per_minute: f64) -> Self {
        let minutes_per_beat = 1_f64 / beats_per_minute;
        let seconds_per_beat = minutes_per_beat * SECONDS_PER_MINUTE as f64;
        let nanos_per_beat = seconds_per_beat * NANOS_PER_SECOND as f64;
        Self::new(nanos_per_beat as u64)
    }

    pub fn to_beats_per_minute (&self) -> f64 {
        let nanos_per_beat = self.nanos_per_beat;
        let beats_per_nano = 1_f64 / self.nanos_per_beat as f64;
        let beats_per_second = beats_per_nano * NANOS_PER_SECOND as f64;
        let beats_per_minute = beats_per_second * SECONDS_PER_MINUTE as f64;
        beats_per_minute
    }

    pub fn nanos_per_tick (&self) -> u64 {
        (self.nanos_per_beat / self.ticks_per_beat) as u64
    }

    pub fn nanos_per_beat (&self) -> u64 {
        self.nanos_per_beat
    }

    pub fn nanos_per_bar (&self) -> u64 {
        self.nanos_per_beat() * self.beats_per_bar
    }

    pub fn nanos_per_loop (&self) -> u64 {
        self.nanos_per_bar() * self.bars_per_loop
    }

    pub fn nanos_to_ticks (&self, nanos: Nanos) -> u64 {
        (nanos / self.nanos_per_tick()) % self.ticks_per_beat
    }

    pub fn nanos_to_beats (&self, nanos: Nanos) -> u64 {
        (nanos / self.nanos_per_beat()) % self.beats_per_bar
    }

    pub fn nanos_to_bars (&self, nanos: Nanos) -> u64 {
        nanos / self.nanos_per_bar() % self.bars_per_loop
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Time {
    nanos: Nanos,
    signature: Signature
}

impl Time {
    pub fn new (nanos: Nanos, signature: Signature) -> Self {
        Self {
            nanos,
            signature
        }
    }

    pub fn nanos (&self) -> Nanos {
        self.nanos
    }

    pub fn ticks (&self) -> Ticks {
        self.signature.nanos_to_ticks(self.nanos)
    }

    pub fn beats (&self) -> Beats {
        self.signature.nanos_to_beats(self.nanos)
    }

    pub fn bars (&self) -> Bars {
        self.signature.nanos_to_bars(self.nanos)
    }

    pub fn nanos_since_loop (&self) -> Nanos {
        self.nanos % self.signature.nanos_per_loop()
    }

    pub fn nanos_since_tick (&self) -> Nanos {
        self.nanos % self.signature.nanos_per_tick()
    }

    pub fn nanos_since_beat (&self) -> Nanos {
        self.nanos % self.signature.nanos_per_beat()
    }

    pub fn nanos_since_bar (&self) -> Nanos {
        self.nanos % self.signature.nanos_per_bar()
    }
}

#[derive(Debug)]
pub struct Clock {
    tick: Instant,
    tap: Option<Instant>,
    nanos: Nanos,
    signature: Signature
}

pub enum Message {
    NudgeTempo(f64),
    Reset,
    Signature(Signature),
    Tap,
}

impl Clock {
    pub fn new () -> Self {
        let tick = Instant::now();
        let signature = Signature::default();
        
        Self {
            nanos: 0,
            tap: None,
            tick,
            signature
        }
    }

    pub fn start (metronome_tx: Sender<metronome::Message>) -> Sender<Message> {
        let mut clock = Self::new();

        let (tx, rx) = channel();

        metronome_tx.send(metronome::Message::Signature(Signature::from_beats_per_minute(DEFAULT_BEATS_PER_MINUTE))).unwrap();

        spawn(move|| {
            loop {
                // wait a tick
                let diff = clock.tick();

                // send clock time
                metronome_tx.send(metronome::Message::Time(clock.time())).unwrap();

                // handle any incoming messages
                let mut is_empty = false;
                while !is_empty {
                    let message_result = rx.try_recv();
                    match message_result {
                        Ok(Message::Reset) => {
                            clock.reset();
                        },
                        Ok(Message::Signature(signature)) => {
                            clock.signature = signature;
                        },
                        Ok(Message::Tap) => {
                            // find how far off the beat we are
                            let time = clock.time();
                            let nanos_since_beat = time.nanos_since_beat();
                            let nanos_per_beat = time.signature.nanos_per_beat();
                            let nanos_per_half_beat = time.signature.nanos_per_beat() / 2;
                            // if the beat happened recently
                            if nanos_since_beat < nanos_per_half_beat {
                                // nudge back to the beat
                                clock.nanos = time.nanos - nanos_since_beat
                            } else {
                                // nudge to the next beat
                                clock.nanos = time.nanos + nanos_per_beat - nanos_since_beat
                            }

                            // if second tap on beat, adjust tempo
                            match clock.tap {
                                Some(tap) => {
                                    let tap_diff = duration_to_nanos(tap.elapsed());
                                    if tap_diff < (nanos_per_beat * 2) {
                                        let next_signature = Signature::new(tap_diff);
                                        metronome_tx.send(metronome::Message::Signature(next_signature));
                                    }
                                },
                                None => {}
                            }

                            clock.tap = Some(Instant::now());
                        },
                        Ok(Message::NudgeTempo(nudge)) => {
                            let old_beats_per_minute = clock.signature.to_beats_per_minute();
                            let new_beats_per_minute = old_beats_per_minute + nudge;
                            let next_signature = Signature::from_beats_per_minute(new_beats_per_minute);
                            metronome_tx.send(metronome::Message::Signature(next_signature));
                        },
                        Err(TryRecvError::Empty) => {
                            is_empty = true;
                        },
                        Err(TryRecvError::Disconnected) => {
                            panic!("{:?}", TryRecvError::Disconnected);
                        }
                    }
                }
            }
        });

        tx
    }

    pub fn reset (&mut self) {
        self.nanos = 0;
        self.tick = Instant::now();
        self.tap = None;
    }

    pub fn time (&self) -> Time {
        Time::new(self.nanos_since_loop(), self.signature)
    }

    pub fn nanos_since_loop (&self) -> Nanos {
        self.nanos % self.signature.nanos_per_loop()
    }

    pub fn nanos_since_tick (&self) -> Nanos {
        duration_to_nanos(self.tick.elapsed())  % self.signature.nanos_per_tick()
    }

    pub fn nanos_until_tick (&self) -> Nanos {
        let nanos_since_tick = self.nanos_since_tick();
        let nanos_per_tick = self.signature.nanos_per_tick();
        nanos_per_tick - nanos_since_tick
    }

    // https://github.com/BookOwl/fps_clock/blob/master/src/lib.rs
    pub fn tick (&mut self) -> Nanos {
        let nanos_until_tick = self.nanos_until_tick();

        sleep(Duration::new(0, nanos_until_tick as u32));

        self.nanos = self.nanos + nanos_until_tick;
        self.tick = Instant::now();

        nanos_until_tick
    }
}

fn duration_to_nanos (duration: Duration) -> Nanos {
    duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as Nanos
}

/*
pub fn nanos_from_ticks (ticks: Ticks, signature: Signature) -> Nanos {
    ticks * signature.nanos_per_beat
}

pub fn ticks_from_beats (beats: Beats, signature: Signature) -> Ticks {
    beats * signature.ticks_per_beat
}

pub fn ticks_from_measure (measures: Measures, signature: Signature) -> Ticks {
    measures * signature.beats_per_measure * signature.ticks_per_beat
}
*/
