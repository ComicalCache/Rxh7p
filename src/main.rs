#![feature(iter_map_windows)]

use std::{sync::mpsc, thread};

use chess::Board;

use crate::{
    engine::Engine,
    uci::{Uci, UciCommand},
};

mod engine;
mod evaluator;
mod sorter;
mod transposition_table;
mod uci;

fn main() {
    let (uci_tx, uci_rx) = mpsc::channel::<UciCommand>();
    let (stop_tx, stop_rx) = mpsc::channel::<()>();
    let (ponderhit_tx, ponderhit_rx) = mpsc::channel::<()>();
    let mut engine = Engine::new(stop_rx, ponderhit_rx);

    let uci_thread = thread::spawn(|| {
        Uci::new(uci_tx, stop_tx, ponderhit_tx).start();
    });

    // No need to receive quit command as this loop stops when the tx value gets dropped.
    while let Ok(command) = uci_rx.recv() {
        match command {
            UciCommand::Invalid => unreachable!("Received unreachable command"),
            UciCommand::Uci => unreachable!("Received uci command"),
            UciCommand::IsReady => Uci::ok(),
            UciCommand::UciNewGame => engine.init(Board::default()),
            UciCommand::Position(board, moves) => engine.uci_position(board, moves),
            UciCommand::Go(config) => engine.go(config),
            UciCommand::Stop => unreachable!("Received stop command"),
            UciCommand::Ponderhit => unreachable!("Received ponderhit command"),
            UciCommand::Quit => unreachable!("Received quit command"),
        }
    }

    if let Err(err) = uci_thread.join() {
        println!("Failed to join uci thread: {err:#?}");
    }
}
