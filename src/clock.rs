// inspired by https://github.com/mmckegg/rust-loop-drop/blob/master/src/midi_time.rs
// http://www.deluge.co/?q=midi-tempo-bpm

use std::time::{Duration, Instant};
use std::thread::{sleep};

pub type Time = Instant;

pub type Nanos = u64;
pub type Ticks = u64;
pub type Beats = u64;
pub type Measures = u64;

static SECONDS_PER_MINUTE: u64 = 60;
static NANOS_PER_SECOND: u64 = 1_000_000;
static BEATS_PER_MINUTE: u64 = 60;
static DEFAULT_TICKS_PER_BEAT: u64 = 1;
static DEFAULT_BEATS_PER_MEASURE: u64 = 4;

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct TimeSignature {
    nanos_per_beat: u64, // tempo
    clicks_per_beat: u64, // meter: notes per beat
    beats_per_bar: u64 // meter
}

impl TimeSignature {
    pub fn new (beats_per_minute: f64) {
        let beats_per_nano = ticks_per_minute 
    }

    pub fn nanos_per_tick (&self) -> u64 {
        (self.nanos_per_beat / self.ticks_per_beat) as u64
    }

    pub fn ticks_since (&self, diff_duration: Duration) -> u64 {
        let total_nanos = diff_duration.as_secs() * 1_000_000_000 + diff_duration.subsec_nanos() as u64;
        self.nanos_per_tick() - total_nanos
    }
}

#[derive(Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Debug, Hash)]
pub struct Clock {
    time: Time,
    time_signature: TimeSignature
}

pub struct ClockMessage {
}

impl Clock {
    pub fn new (time_signature: TimeSignature) -> Self {
        Self {
            time: Instant::now(),
            time_signature
        }
    }

    pub fn get_time (&self) -> Time {
        self.time
    }

    // https://github.com/BookOwl/fps_clock/blob/master/src/lib.rs
    pub fn tick (&mut self) -> u64 {
        let diff_duration = self.time.elapsed();
        let diff_nanos = self.time_signature.nanos_since(diff_duration);
        if diff_nanos > 0 {
            sleep(Duration::new(0, diff_nanos as u32))
        };
        self.time = Instant::now();
        diff_nanos
    }
}

/*
pub fn nanos_from_ticks (ticks: Ticks, time_signature: TimeSignature) -> Nanos {
    ticks * time_signature.nanos_per_beat
}

pub fn ticks_from_beats (beats: Beats, time_signature: TimeSignature) -> Ticks {
    beats * time_signature.ticks_per_beat
}

pub fn ticks_from_measure (measures: Measures, time_signature: TimeSignature) -> Ticks {
    measures * time_signature.beats_per_measure * time_signature.ticks_per_beat
}
*/
