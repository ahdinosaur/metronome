// inspired by https://github.com/mmckegg/rust-loop-drop/blob/master/src/midi_time.rs
// http://www.deluge.co/?q=midi-tempo-bpm

use std::time::{Duration, Instant};
use std::thread::{sleep};

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

#[derive(Clone, Copy, Debug, Hash)]
pub struct ClockSignature {
    nanos_per_beat: u64, // tempo
    ticks_per_beat: u64, // meter
    beats_per_bar: u64 // meter
}

impl ClockSignature {
    pub fn new (beats_per_minute: f64) -> Self {
        let minutes_per_beat = 1_f64 / beats_per_minute;
        let seconds_per_beat = minutes_per_beat * SECONDS_PER_MINUTE as f64;
        let nanos_per_beat = seconds_per_beat * NANOS_PER_SECOND as f64;

        Self {
            nanos_per_beat: nanos_per_beat as u64,
            ticks_per_beat: DEFAULT_TICKS_PER_BEAT,
            beats_per_bar: DEFAULT_BEATS_PER_BAR
        }
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
        (nanos / self.nanos_per_tick()) as u64
    }

    pub fn nanos_to_beats (&self, nanos: Nanos) -> u64 {
        (nanos / self.nanos_per_beat()) as u64
    }

    pub fn nanos_to_bars (&self, nanos: Nanos) -> u64 {
        (nanos / self.nanos_per_bar()) as u64
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct ClockTime {
    nanos: Nanos,
    ticks: Ticks,
    beats: Beats,
    bars: Bars
}

impl ClockTime {
    pub fn new (nanos: Nanos, signature: ClockSignature) -> Self {
        Self {
            nanos,
            ticks: signature.nanos_to_ticks(nanos),
            beats: signature.nanos_to_beats(nanos),
            bars: signature.nanos_to_bars(nanos)
        }
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub struct Clock {
    start_instant: Instant,
    tick_instant: Instant,
    signature: ClockSignature
}

pub struct ClockMessage {
}

impl Clock {
    pub fn new (signature: ClockSignature) -> Self {
        let start_instant = Instant::now();

        Self {
            start_instant,
            tick_instant: start_instant,
            signature
        }
    }

    pub fn time (&self) -> ClockTime {
        ClockTime::new(self.nanos_since_start(), self.signature)
    }

    pub fn diff (&self) -> ClockTime {
        ClockTime::new(self.nanos_since_tick(), self.signature)
    }
    
    pub fn nanos_since_start (&self) -> Nanos {
        duration_to_nanos(self.start_instant.elapsed())
    }

    pub fn nanos_since_tick (&self) -> Nanos {
        duration_to_nanos(self.tick_instant.elapsed())
    }

    // https://github.com/BookOwl/fps_clock/blob/master/src/lib.rs
    pub fn tick (&mut self) -> ClockTime {
        let diff = self.diff();

        if diff.nanos > 0 {
            sleep(Duration::new(0, diff.nanos as u32))
        };


        self.tick_instant = Instant::now();

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
