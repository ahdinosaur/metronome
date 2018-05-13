extern crate ncurses;

mod clock;
mod control;
mod interface;

fn main () {
    Metronome::run(60_f64);
}


pub type Tempo = f64;

struct Metronome {}

impl Metronome {
    pub fn run (tempo: Tempo) {
        let control = control::Control::new();

        let terminal_tx = interface::TerminalInterface::start(control.tx.clone());
        let clock_tx = clock::Clock::start(control.tx.clone());

        for control_message in control.rx {
            match control_message {
                // sent by interface
                control::ControlMessage::Reset => {
                    clock_tx.send(clock::ClockMessage::Reset).unwrap();
                },
                // sent by interface
                control::ControlMessage::NudgeTempo(nudge) => {
                    clock_tx.send(clock::ClockMessage::NudgeTempo(nudge)).unwrap();
                },
                // sent by clock
                control::ControlMessage::Signature(signature) => {
                    clock_tx.send(clock::ClockMessage::Signature(signature)).unwrap();
                    terminal_tx.send(interface::InterfaceMessage::Signature(signature)).unwrap();
                },
                // sent by clock
                control::ControlMessage::Time(time) => {
                    terminal_tx.send(interface::InterfaceMessage::Time(time)).unwrap();
                },
                _ => {}
            }
        }
    }
}

