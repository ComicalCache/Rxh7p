#![feature(iter_map_windows)]

use std::{str::FromStr, time::Duration};

use chess::{Board, ChessMove};

use crate::engine::Engine;

mod engine;
mod evaluator;
mod io;
mod sorter;
mod transposition_table;

const MAX_DEPTH: u16 = 50;

fn main() {
    printf!(" Initial game state: ");
    let mut fen: String = String::new();
    if std::io::stdin().read_line(&mut fen).is_err() {
        println!(" Failed to read FEN");
        return;
    }
    let fen = fen.trim();

    let board = match Board::from_str(fen) {
        Ok(board) => board,
        _ => {
            println!(" '{fen}' is an invalid FEN string");
            return;
        }
    };
    let mut engine = Engine::new(board);

    let mut ply = 0;
    loop {
        // Search.
        let (eval, depth) = engine.iterative_deepening(MAX_DEPTH, Duration::new(5, 0));

        // Print search results.
        print_results(&engine, ply, depth as u64, eval);

        // Make top engine move.
        if make_move(&mut engine, ply).is_none() {
            println!(" Failed to find next move for position '{}'", engine.board);
            return;
        }

        if has_game_ended(&engine) {
            return;
        }

        // Fetch next opponent move.
        if make_opponent_move(&mut engine).is_err() {
            return;
        }

        if has_game_ended(&engine) {
            return;
        }

        // Increment ply for own and opponent move.
        ply += 2;
    }
}

fn print_results(engine: &Engine, ply: u64, depth: u64, eval: i64) {
    let mut sequence = Vec::new();
    let mut temp_board = engine.board;

    let mut idx = 0;
    while let Some(entry) = engine.tt.get(temp_board.get_hash() + ply + idx)
        && let Some(mv) = entry.mv
        && idx < depth
    {
        sequence.push(format!("{mv}"));
        temp_board = temp_board.make_move_new(mv);

        idx += 1;
    }

    println!(" [EVAL={eval}]: {}", sequence.join(" -> "));
}

fn has_game_ended(engine: &Engine) -> bool {
    match engine.board.status() {
        chess::BoardStatus::Ongoing => false,
        chess::BoardStatus::Stalemate => {
            println!(" Stalemate");
            true
        }
        chess::BoardStatus::Checkmate => {
            println!(" Checkmate");
            true
        }
    }
}

fn make_move(engine: &mut Engine, ply: u64) -> Option<()> {
    let mv = engine.tt.get(engine.board.get_hash() + ply)?.mv?;

    // FIXME: this should be in SAN.
    println!(" Playing {mv}");
    engine.make_move(mv);

    Some(())
}

fn make_opponent_move(engine: &mut Engine) -> Result<(), ()> {
    loop {
        printf!(" Enter opponent move: ");
        let mut mv: String = String::new();
        if std::io::stdin().read_line(&mut mv).is_err() {
            println!(" Failed to read opponent move");
            return Err(());
        }
        let mv = mv.trim();

        if let Ok(mv) = ChessMove::from_san(&engine.board, mv) {
            engine.make_move(mv);
            break;
        }

        println!(" '{mv}' is an invalid SAN move for the current board");
    }

    Ok(())
}
