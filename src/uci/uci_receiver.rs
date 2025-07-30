use std::sync::mpsc::Sender;

use crate::uci::{UciCommand, uci_sender};

pub enum UciSearchStop {
    Ponderhit,
    Stop,
}

/// Struct acting as a UCI receiver. It runs in its own thread and communicates with the engine via
/// message passing. The stop and ponderhit command require their own channel since they need to be
/// receivable by the engine while searching.
pub struct UciReceiver {
    /// Channel for sending received commands to the engine.
    cmd_tx: Sender<UciCommand>,
    /// Channel for sending the search stop commands to the engine.
    search_stop_tx: Sender<UciSearchStop>,
}

impl UciReceiver {
    /// Creates a new UCI receiver.
    pub fn new(cmd_tx: Sender<UciCommand>, search_stop_tx: Sender<UciSearchStop>) -> Self {
        UciReceiver {
            cmd_tx,
            search_stop_tx,
        }
    }

    /// Main loop, reading UCI commands and propagating them to the engine.
    pub fn start(&mut self) {
        let stdin = std::io::stdin();

        loop {
            let mut input = String::new();
            stdin
                .read_line(&mut input)
                .expect("Failed to read from stdin");

            let command = UciCommand::from(input.trim());

            let send = |cmd| {
                self.cmd_tx
                    .send(cmd)
                    .expect("Failed to send command to main thread");
            };

            match command {
                // Don't propagate invalid commands to the engine.
                UciCommand::Invalid => {}
                UciCommand::Uci => uci_sender::id(),
                // Send here since main is busy.
                UciCommand::Stop => self
                    .search_stop_tx
                    .send(UciSearchStop::Stop)
                    .expect("Failed to send stop message to engine"),
                UciCommand::Ponderhit => self
                    .search_stop_tx
                    .send(UciSearchStop::Ponderhit)
                    .expect("Failed to send ponderhit message to engine"),
                // Send stop in case the engine is searching. The loop in main quits when this loop
                // ends since the tx value gets dropped.
                UciCommand::Quit => {
                    self.search_stop_tx
                        .send(UciSearchStop::Stop)
                        .expect("Failed to send stop message to engine");
                    break;
                }
                _ => send(command),
            }
        }
    }
}
