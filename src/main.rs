#![feature(duration_millis_float)]

#[cfg(feature = "logging")]
use std::{env, fs::File, io::Write};

use std::{sync::mpsc, thread};

use chess::Board;

use crate::{
    engine::Engine,
    uci::{UciCommand, UciReceiver, UciSearchStop, uci_sender},
};

mod engine;
mod evaluator;
mod orderer;
mod tt;
mod uci;

fn main() {
    #[cfg(feature = "logging")]
    let log_path = env::args()
        .skip(1)
        .next()
        .expect("Expected log file path in logging build.");

    let (uci_receiver_tx, uci_receivre_rx) = mpsc::channel::<UciCommand>();
    let (search_stop_tx, search_stop_rx) = mpsc::channel::<UciSearchStop>();
    let mut engine = Engine::new(search_stop_rx);

    let uci_receiver_thread = thread::spawn(|| {
        UciReceiver::new(uci_receiver_tx, search_stop_tx).start();
    });

    while let Ok(command) = uci_receivre_rx.recv() {
        match command {
            // Ignored by the receiver.
            UciCommand::Invalid => unreachable!("Received unreachable command"),
            // Simple handshake by the receiver.
            UciCommand::Uci => unreachable!("Received uci command"),
            UciCommand::IsReady => uci_sender::ready_ok(),
            UciCommand::UciNewGame => engine.uci_init(Board::default()),
            UciCommand::Position(board, moves) => engine.uci_position(board, moves),
            UciCommand::Go(config) => engine.go(config),
            // The receiver sends a message via MPSC channels.
            UciCommand::Stop => unreachable!("Received stop command"),
            // The receiver sends a message via MPSC channels.
            UciCommand::Ponderhit => unreachable!("Received ponderhit command"),
            // No need to receive quit command as this loop stops when the tx value gets dropped.
            // The receiver also sends a stop message via MPSC channels to cancel any ongoing
            // searches.
            UciCommand::Quit => unreachable!("Received quit command"),
        }
    }

    if let Err(err) = uci_receiver_thread.join() {
        println!("Failed to join uci receiver thread: {err:#?}");
    }

    #[cfg(feature = "logging")]
    write_logs(log_path, engine);
}

#[cfg(feature = "logging")]
fn write_logs(log_path: String, engine: Engine) {
    let mut log_file = match File::create(log_path) {
        Ok(file) => file,
        Err(err) => panic!("Failed to create log file: {err}"),
    };

    if let Err(err) = writeln!(
        &mut log_file,
        "=== START LOG ===\nply,game phase,soft move time,hard move time,time limit kind"
    ) {
        panic!("Failed to write to log file: {err}");
    }
    for entry in engine.search_log {
        if let Err(err) = writeln!(&mut log_file, "{entry}") {
            panic!("Failed to write to log file: {err}");
        }
    }
    if let Err(err) = writeln!(&mut log_file, "=== END LOG ===") {
        panic!("Failed to write to log file: {err}");
    }
}
