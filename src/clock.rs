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

pub type Tempo = f64;

static SECONDS_PER_MINUTE: u64 = 60;
static NANOS_PER_SECOND: u64 = 1_000_000_000;
static BEATS_PER_MINUTE: u64 = 60;
static DEFAULT_TICKS_PER_BEAT: u64 = 16;
static DEFAULT_BEATS_PER_BAR: u64 = 4;
static DEFAULT_BARS_PER_LOOP: u64 = 4;
static DEFAULT_BEATS_PER_MINUTE: f64 = 60_f64;

#[derive(Clone, Copy, Debug)]
pub struct Signature {
    pub ticks_per_beat: u64, // meter
    pub beats_per_bar: u64, // meter
    pub bars_per_loop: u64,
}

impl Signature {
    pub fn default () -> Self {
        Self {
            ticks_per_beat: DEFAULT_TICKS_PER_BEAT,
            beats_per_bar: DEFAULT_BEATS_PER_BAR,
            bars_per_loop: DEFAULT_BARS_PER_LOOP,
        }
    }

    pub fn ticks_per_beat (&self) -> Ticks {
        self.ticks_per_beat
    }

    pub fn ticks_per_bar (&self) -> Ticks {
        self.ticks_per_beat() * self.beats_per_bar
    }

    pub fn ticks_per_loop (&self) -> Ticks {
        self.ticks_per_bar() * self.bars_per_loop
    }

    pub fn ticks_to_beats (&self, ticks: Ticks) -> f64 {
        ticks as f64 / self.ticks_per_beat as f64
    }

    pub fn ticks_to_bars (&self, ticks: Ticks) -> f64 {
        self.ticks_to_beats(ticks) / self.beats_per_bar as f64
    }

    pub fn ticks_to_loops (&self, ticks: Ticks) -> f64 {
        self.ticks_to_bars(ticks) / self.bars_per_loop as f64
    }

    pub fn nanos_per_tick (&self, beats_per_minute: f64) -> Nanos {
        let minutes_per_beat = 1_f64 / beats_per_minute;
        let seconds_per_beat = minutes_per_beat * SECONDS_PER_MINUTE as f64;
        let nanos_per_beat = seconds_per_beat * NANOS_PER_SECOND as f64;
        let nanos_per_tick = nanos_per_beat / self.ticks_per_beat as f64;
        nanos_per_tick as Nanos
    }

    pub fn nanos_per_beat (&self, beats_per_minute: f64) -> Nanos {
        self.nanos_per_tick(beats_per_minute) * self.ticks_per_beat
    }

    pub fn nanos_per_bar (&self, beats_per_minute: f64) -> Nanos {
        self.nanos_per_beat(beats_per_minute) * self.beats_per_bar
    }

    pub fn nanos_per_loop (&self, beats_per_minute: f64) -> Nanos {
        self.nanos_per_bar(beats_per_minute) * self.bars_per_loop
    }

    pub fn beats_per_minute (&self, nanos_per_tick: f64) -> Tempo {
        let nanos_per_beat = nanos_per_tick * self.ticks_per_beat as f64;
        let beats_per_nano = 1_f64 / nanos_per_beat as f64;
        let beats_per_second = beats_per_nano * NANOS_PER_SECOND as f64;
        let beats_per_minute = beats_per_second * SECONDS_PER_MINUTE as f64;
        beats_per_minute
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Time {
    ticks: Ticks,
    signature: Signature
}

impl Time {
    pub fn new (signature: Signature) -> Self {
        Self {
            ticks: 0,
            signature
        }
    }

    pub fn ticks (&self) -> Ticks {
        self.ticks
    }

    pub fn beats (&self) -> f64 {
        self.signature.ticks_to_beats(self.ticks)
    }

    pub fn bars (&self) -> f64 {
        self.signature.ticks_to_bars(self.ticks)
    }

    pub fn loops (&self) -> f64 {
        self.signature.ticks_to_loops(self.ticks)
    }

    pub fn next (&self) -> Self {
        Self {
            ticks: self.ticks + 1,
            signature: self.signature
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Timer {
    instant: Instant,
    signature: Signature
}

impl Timer {
    pub fn new (signature: Signature) -> Self {
        Self {
            instant: Instant::now(),
            signature
        }
    }

    pub fn nanos (&self) -> Nanos {
        duration_to_nanos(self.instant.elapsed())
    }

    pub fn nanos_since_tick (&self, beats_per_minute: f64) -> Nanos {
        self.nanos() % self.signature.nanos_per_tick(beats_per_minute)
    }

    pub fn nanos_since_beat (&self, beats_per_minute: f64) -> Nanos {
        self.nanos() % self.signature.nanos_per_beat(beats_per_minute)
    }

    pub fn nanos_since_bar (&self, beats_per_minute: f64) -> Nanos {
        self.nanos() % self.signature.nanos_per_bar(beats_per_minute)
    }

    pub fn nanos_since_loop (&self, beats_per_minute: f64) -> Nanos {
        self.nanos() % self.signature.nanos_per_loop(beats_per_minute)
    }

    pub fn nanos_until_tick (&self, beats_per_minute: f64) -> Nanos {
        let nanos_since_tick = self.nanos_since_tick(beats_per_minute);
        let nanos_per_tick = self.signature.nanos_per_tick(beats_per_minute);
        nanos_per_tick - nanos_since_tick
    }

    pub fn next (&self, beats_per_minute: f64) -> Nanos {
        let nanos_until_tick = self.nanos_until_tick(beats_per_minute);

        sleep(Duration::new(0, nanos_until_tick as u32));

        nanos_until_tick
    }
}

#[derive(Debug)]
pub struct Clock {
    time: Time,
    timer: Timer,
    signature: Signature,
    tempo: Tempo,
    tap: Option<Instant>
}

pub enum Message {
    Tempo(Tempo),
    NudgeTempo(f64),
    Reset,
    Signature(Signature),
    Tap,
}

impl Clock {
    pub fn new () -> Self {
        let signature = Signature::default();
        let time = Time::new(signature);
        let timer = Timer::new(signature);
        let tempo = DEFAULT_BEATS_PER_MINUTE;
        
        Self {
            time,
            timer,
            signature,
            tempo,
            tap: None
        }
    }

    pub fn start (metronome_tx: Sender<metronome::Message>) -> Sender<Message> {
        let mut clock = Self::new();

        let (tx, rx) = channel();

        metronome_tx.send(metronome::Message::Signature(Signature::default())).unwrap();
        metronome_tx.send(metronome::Message::Tempo(clock.tempo)).unwrap();

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
                            clock.set_signature(signature);
                        },
                        Ok(Message::Tap) => {
                            /*
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
                            */
                        },
                        Ok(Message::NudgeTempo(nudge)) => {
                            let old_tempo = clock.tempo;
                            let new_tempo = old_tempo + nudge;
                            metronome_tx.send(metronome::Message::Tempo(new_tempo));
                        },
                        Ok(Message::Tempo(tempo)) => {
                            clock.tempo = tempo;
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
        self.time = Time::new(self.signature);
        self.timer = Timer::new(self.signature);
    }

    pub fn set_signature(&mut self, signature: Signature) {
        self.signature = signature;
        self.time = Time::new(self.signature);
        self.timer = Timer::new(self.signature);
    }

    pub fn time (&self) -> Time {
        self.time
    }

    pub fn tick (&mut self) -> Nanos {
        let nanos_until_tick = self.timer.next(self.tempo);
        self.time = self.time.next();
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
