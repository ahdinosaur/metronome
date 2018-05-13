use std::sync::mpsc::{channel, Sender, Receiver};

use clock;

#[derive(Debug)]
pub struct Control {
    pub tx: Sender<ControlMessage>,
    pub rx: Receiver<ControlMessage>
}

impl Control {
    pub fn new() -> Self {
        let (tx, rx) = channel();
        
        Self {
            tx,
            rx
        }
    }
}

#[derive(Clone, Copy, Debug, Hash)]
pub enum ControlMessage {
    Time(clock::ClockTime),
    Signature(clock::ClockSignature),
    Start,
    Stop,
    TapTempo,
    SetTempo,
    NudgeClock,
    Configure
}

