use std::sync::mpsc::Sender;

use crate::uci::{UciCommand, UciSender};

/// Struct acting as a UCI receiver. It runs in its own thread and communicates with the engine via
/// message passing. The stop and ponderhit command require their own channels since they need to be
/// receivable by the engine while searching.
pub struct UciReceiver {
    /// Channel for sending received commands to the engine.
    cmd_tx: Sender<UciCommand>,
    /// Channel for sending the stop command to the engine.
    stop_tx: Sender<()>,
    /// Channel for sending the ponderhit command to the engine.
    ponderhit_tx: Sender<()>,
}

impl UciReceiver {
    /// Creates a new UCI receiver.
    pub fn new(cmd_tx: Sender<UciCommand>, stop_tx: Sender<()>, ponderhit_tx: Sender<()>) -> Self {
        UciReceiver {
            cmd_tx,
            stop_tx,
            ponderhit_tx,
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
                    .expect("Failed to send command to main thread")
            };

            match command {
                // Don't propagate invalid commands to the engine.
                UciCommand::Invalid => continue,
                UciCommand::Uci => UciSender::id(),
                // Send here since main is busy.
                UciCommand::Stop => self
                    .stop_tx
                    .send(())
                    .expect("Failed to send stop message to engine"),
                UciCommand::Ponderhit => self
                    .ponderhit_tx
                    .send(())
                    .expect("Failed to send ponderhit message to engine"),
                // Send stop in case the engine is searching. The loop in main quits when this loop
                // ends since the tx value gets dropped.
                UciCommand::Quit => {
                    self.stop_tx
                        .send(())
                        .expect("Failed to send stop message to engine");
                    break;
                }
                _ => send(command),
            }
        }
    }
}
