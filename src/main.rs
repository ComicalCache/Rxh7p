use std::{io, str::FromStr};

use chess::Board;

use crate::engine::Engine;

mod engine;
mod evaluator;
mod sorter;
mod transposition_table;

const DEPTH: u16 = 6;

fn main() {
    let mut engine = Engine::new();

    loop {
        let mut fen: String = String::new();
        io::stdin()
            .read_line(&mut fen)
            .expect("Unable to read Stdin");

        let mut board = Board::from_str(&fen).unwrap();
        let side = board.side_to_move();

        engine.alpha_beta(board, i64::MIN + 1, i64::MAX, DEPTH);

        let mut sequence = Vec::new();
        for _ in 0..DEPTH {
            if let Some(entry) = engine.tt.get(board.get_hash()) {
                if let Some(mv) = entry.mv {
                    // Always print from view of who's current turn it is to avoid the evaluation
                    // jumping from plus to minus in pv-search.
                    sequence.push(format!("[{}] {mv}", entry.eval(side)));
                    board = board.make_move_new(mv);
                } else {
                    break;
                }
            }
        }

        println!("{}", sequence.join(" -> "));
    }
}
