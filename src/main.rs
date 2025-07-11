use std::{io, str::FromStr};

use chess::Board;

use crate::searcher::Searcher;

mod evaluator;
mod searcher;
mod sorter;
mod transposition_table;

const DEPTH: u16 = 7;

fn main() {
    let mut searcher = Searcher::new();

    loop {
        let mut fen: String = String::new();
        io::stdin()
            .read_line(&mut fen)
            .expect("Unable to read Stdin");

        let mut board = Board::from_str(&fen).unwrap();
        let side = board.side_to_move();

        searcher.alpha_beta(board, i64::MIN + 1, i64::MAX, DEPTH);

        let mut sequence = Vec::new();
        for _ in 0..DEPTH {
            if let Some(entry) = searcher.tt.get(board.get_hash()) {
                if let Some(mv) = entry.mv {
                    // always print from view of who's current turn it is
                    // avoids eval to jump from plus to minus in pv
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
