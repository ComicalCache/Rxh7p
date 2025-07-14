use std::{i64, io, str::FromStr};

use chess::Board;

use crate::engine::Engine;

mod engine;
mod evaluator;
mod sorter;
mod transposition_table;

const DEPTH: u16 = 4;

fn main() {
    let mut engine = Engine::new();

    loop {
        let mut fen: String = String::new();
        io::stdin()
            .read_line(&mut fen)
            .expect("Unable to read Stdin");

        let mut board = Board::from_str(&fen).unwrap();

        let _ = engine.negamax(board, i64::MIN + 1, i64::MAX, DEPTH, DEPTH);

        let mut sequence = Vec::new();
        for _ in 0..DEPTH {
            let entry = engine.tt.get(board.get_hash());
            if let Some(mv) = entry.mv {
                sequence.push(format!("{}: {mv}", entry.eval(board.side_to_move())));
                board = board.make_move_new(mv);
            } else {
                break;
            }
        }

        println!("{}", sequence.join(" -> "));
    }
}
