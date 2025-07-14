use std::{io, str::FromStr, time::Duration};

use chess::Board;

use crate::engine::Engine;

mod engine;
mod evaluator;
mod sorter;
mod transposition_table;

const DEPTH: u16 = 10;

fn main() {
    let mut engine = Engine::new();

    loop {
        let mut fen: String = String::new();
        io::stdin()
            .read_line(&mut fen)
            .expect("Unable to read Stdin");

        let mut board = Board::from_str(&fen).unwrap();
        let eval = engine.iterative_deepening(board, DEPTH, Duration::new(5, 0));

        let mut sequence = Vec::new();
        for _ in 0..DEPTH {
            let entry = engine.tt.get(board.get_hash());
            assert!(
                entry.hash == board.get_hash(),
                "Transposition table didn't contain searched position to display move to make."
            );

            if let Some(mv) = entry.mv {
                sequence.push(format!("{mv}"));
                board = board.make_move_new(mv);
            } else {
                break;
            }
        }

        println!("[{eval}]: {}", sequence.join(" -> "));
    }
}
