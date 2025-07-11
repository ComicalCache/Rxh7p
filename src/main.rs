use std::{i64, io, str::FromStr};

use chess::Board;

use crate::searcher::Searcher;

mod evaluator;
mod searcher;
mod sorter;
mod transposition_table;

fn main() {
    let mut searcher = Searcher::new();

    loop {
        let mut fen: String = String::new();
        io::stdin()
            .read_line(&mut fen)
            .expect("Unable to read Stdin");

        let board = Board::from_str(&fen).unwrap();

        let _ = searcher.alpha_beta(board, i64::MIN + 1, i64::MAX, 4, 4);
        let entry = searcher.tt.get(board.get_hash());

        println!(
            "{}: {}",
            entry.eval(board.side_to_move()),
            entry.mv.unwrap()
        );
    }
}
