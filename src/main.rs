use std::{i64, io, str::FromStr};

use chess::Board;

use crate::searcher::Searcher;

mod engine;
mod evaluator;
mod searcher;
mod sorter;

fn main() {
    loop {
        let mut fen: String = String::new();
        io::stdin()
            .read_line(&mut fen)
            .expect("Unable to read Stdin");

        let board = Board::from_str(&fen).unwrap();

        let (score, m) = Searcher::alpha_beta(board, i64::MIN + 1, i64::MAX, 4);

        println!(
            "{score}: {}",
            if m.is_some() {
                m.unwrap().to_string()
            } else {
                "None".to_string()
            }
        );
    }
}
