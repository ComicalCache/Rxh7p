#![feature(iter_map_windows)]

use std::{io, str::FromStr};

use chess::Board;

use crate::{engine::Engine, evaluator::Evaluator};

mod engine;
mod evaluator;
mod sorter;
mod transposition_table;

const DEPTH: u16 = 8;

fn main() {
    let mut engine = Engine::new();

    loop {
        let mut fen: String = String::new();
        io::stdin()
            .read_line(&mut fen)
            .expect("Unable to read Stdin");

        let mut board = Board::from_str(&fen).unwrap();

        if false {
            let mut eval = Evaluator::new();
            println!("Evaluation: {}", eval.evaluate(board));
            continue;
        }

        engine.alpha_beta(&board, i64::MIN + 1, i64::MAX, DEPTH);

        let mut sequence = Vec::new();
        for idx in 0..DEPTH {
            if let Some(entry) = engine.tt.get(board.get_hash()) {
                if let Some(mv) = entry.mv {
                    if idx == 0 {
                        sequence.push(format!("{}", entry.eval(board.side_to_move())));
                    }

                    // Always print from view of who's current turn it is to avoid the evaluation
                    // jumping from plus to minus in pv-search.
                    sequence.push(format!("{mv}"));
                    board = board.make_move_new(mv);
                } else {
                    break;
                }
            }
        }

        println!(
            "[{}] {}",
            sequence.first().unwrap_or(&"No eval".to_string()),
            sequence[1..].join(" -> ")
        );
    }
}
