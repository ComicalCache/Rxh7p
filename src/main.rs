#![feature(iter_map_windows)]

use std::{sync::mpsc, thread};

use chess::Board;

use crate::{
    engine::Engine,
    uci::{UciCommand, UciReceiver, UciSender, UciSenderMessage},
};

mod engine;
mod evaluator;
mod orderer;
mod tt;
mod uci;

fn main() {
    let (uci_receiver_tx, uci_receivre_rx) = mpsc::channel::<UciCommand>();
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (ponderhit_tx, ponderhit_rx) = mpsc::channel::<()>();
    let (uci_sender_tx, uci_sender_rx) = mpsc::channel::<UciSenderMessage>();
    let mut engine = Engine::new(uci_sender_tx, stop_rx, ponderhit_rx);

    let uci_receiver_thread = thread::spawn(|| {
        UciReceiver::new(uci_receiver_tx, stop_tx, ponderhit_tx).start();
    });
    let uci_sender_thread = thread::spawn(|| {
        UciSender::new(uci_sender_rx).start();
    });

    while let Ok(command) = uci_receivre_rx.recv() {
        match command {
            // Ignored by the receiver.
            UciCommand::Invalid => unreachable!("Received unreachable command"),
            // Simple handshake by the receiver.
            UciCommand::Uci => unreachable!("Received uci command"),
            UciCommand::IsReady => UciSender::ready_ok(),
            UciCommand::UciNewGame => engine.init(Board::default()),
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

    // This drops the uci sender tx and thus stops the loop in UciSender::start, causing the thread
    // to stop.
    drop(engine);

    if let Err(err) = uci_sender_thread.join() {
        println!("Failed to join uci sender thread: {err:#?}");
    }
}
