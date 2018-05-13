extern crate ncurses;

mod clock;
mod control;
mod interface;

fn main () {
    Metronome::run(60_f64);
}


pub type Bpm = f64;

struct Metronome {
    pub bpm: Bpm
}

impl Metronome {
    pub fn run (bpm: Bpm) {
        let control = control::Control::new();

        let clock_signature = clock::ClockSignature::new(bpm);

        let clock = clock::Clock::start(clock_signature, control.tx.clone());
        let terminal_interface = interface::TerminalInterface::start(clock_signature, control.tx.clone());

        for control_message in control.rx {
            match control_message {
                control::ControlMessage::Signature(signature) => {
                    terminal_interface.tx.send(interface::InterfaceMessage::Signature(signature)).unwrap();
                },
                control::ControlMessage::Time(time) => {
                    terminal_interface.tx.send(interface::InterfaceMessage::Time(time)).unwrap();
                },
                _ => {}
            }
        }
    }
}

