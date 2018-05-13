// inspired by https://github.com/mmckegg/rust-loop-drop/blob/master/src/midi_time.rs
// http://www.deluge.co/?q=midi-tempo-bpm

use std::time::{Duration, Instant};
use std::thread::{sleep, spawn};
use std::sync::mpsc::{channel, Sender, Receiver, TryRecvError};

use control;

pub type Time = Instant;

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

#[derive(Clone, Copy, Debug, Hash)]
pub struct ClockSignature {
    pub nanos_per_beat: u64, // tempo
    pub ticks_per_beat: u64, // meter
    pub beats_per_bar: u64, // meter
    pub bars_per_loop: u64,
}

impl ClockSignature {
    pub fn new (beats_per_minute: f64) -> Self {
        let minutes_per_beat = 1_f64 / beats_per_minute;
        let seconds_per_beat = minutes_per_beat * SECONDS_PER_MINUTE as f64;
        let nanos_per_beat = seconds_per_beat * NANOS_PER_SECOND as f64;

        Self {
            nanos_per_beat: nanos_per_beat as u64,
            ticks_per_beat: DEFAULT_TICKS_PER_BEAT,
            beats_per_bar: DEFAULT_BEATS_PER_BAR,
            bars_per_loop: DEFAULT_BARS_PER_LOOP,
        }
    }

    pub fn default () -> Self {
        Self::new(DEFAULT_BEATS_PER_MINUTE)
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

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct ClockTime {
    pub nanos: Nanos,
    pub ticks: Ticks,
    pub beats: Beats,
    pub bars: Bars
}

impl ClockTime {
    pub fn new (nanos: Nanos, signature: ClockSignature) -> Self {
        Self {
            nanos,
            ticks: signature.nanos_to_ticks(nanos),
            beats: signature.nanos_to_beats(nanos),
            bars: signature.nanos_to_bars(nanos),
    //        ticks_til_beat: signature.ticks_til_beat(nanos),
    //        beats_til_bar: signature.beats_til_bar(nanos)
        }
    }
}

#[derive(Debug)]
pub struct Clock {
    start: Time,
    tick: Time,
    signature: ClockSignature
}

pub enum ClockMessage {
    Reset,
    NudgeTempo(f64),
    Signature(ClockSignature)
}

impl Clock {
    pub fn new () -> Self {
        let start = Time::now();
        let signature = ClockSignature::default();
        
        Self {
            start,
            tick: start,
            signature
        }
    }

    pub fn start (control_tx: Sender<control::ControlMessage>) -> Sender<ClockMessage> {
        let mut clock = Self::new();

        let (tx, rx) = channel();

        control_tx.send(control::ControlMessage::Signature(ClockSignature::new(DEFAULT_BEATS_PER_MINUTE))).unwrap();

        spawn(move|| {
            loop {
                // wait a tick
                let diff = clock.tick();

                // send clock time
                control_tx.send(control::ControlMessage::Time(clock.time())).unwrap();

                // handle any incoming messages
                let message_result = rx.try_recv();
                match message_result {
                    Ok(ClockMessage::Reset) => {
                        clock.reset();
                    },
                    Ok(ClockMessage::Signature(signature)) => {
                        clock.signature = signature;
                    },
                    Ok(ClockMessage::NudgeTempo(nudge)) => {
                        let old_beats_per_minute = clock.signature.to_beats_per_minute();
                        let new_beats_per_minute = old_beats_per_minute - nudge;
                        let next_signature = ClockSignature::new(new_beats_per_minute);
                        control_tx.send(control::ControlMessage::Signature(next_signature));
                    },
                    Err(TryRecvError::Empty) => {},
                    Err(TryRecvError::Disconnected) => {
                        panic!("{:?}", TryRecvError::Disconnected);
                    }
                }
            }
        });

        tx
    }

    pub fn reset (&mut self) {
        self.start = Time::now();
    }

    pub fn time (&self) -> ClockTime {
        ClockTime::new(self.nanos_since_start(), self.signature)
    }

    pub fn diff (&self) -> ClockTime {
        let nanos_since_tick = self.nanos_since_tick();
        let nanos_per_tick = self.signature.nanos_per_tick();
        let diff = nanos_per_tick - nanos_since_tick;
        ClockTime::new(diff, self.signature)
    }
    
    pub fn nanos_since_start (&self) -> Nanos {
        duration_to_nanos(self.start.elapsed())
    }

    pub fn nanos_since_tick (&self) -> Nanos {
        duration_to_nanos(self.tick.elapsed())
    }

    // https://github.com/BookOwl/fps_clock/blob/master/src/lib.rs
    pub fn tick (&mut self) -> ClockTime {
        let diff = self.diff();

        if diff.nanos > 0 {
            sleep(Duration::new(0, diff.nanos as u32))
        };

        self.tick = Time::now();

        diff
    }
}

fn duration_to_nanos (duration: Duration) -> Nanos {
    duration.as_secs() * 1_000_000_000 + duration.subsec_nanos() as Nanos
}

/*
pub fn nanos_from_ticks (ticks: Ticks, signature: ClockSignature) -> Nanos {
    ticks * signature.nanos_per_beat
}

pub fn ticks_from_beats (beats: Beats, signature: ClockSignature) -> Ticks {
    beats * signature.ticks_per_beat
}

pub fn ticks_from_measure (measures: Measures, signature: ClockSignature) -> Ticks {
    measures * signature.beats_per_measure * signature.ticks_per_beat
}
*/
