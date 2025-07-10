use chess::{Board, ChessMove};

use crate::{evaluator::Evaluator, sorter::Sorter};

pub struct Searcher {}

impl Searcher {
    pub fn alpha_beta(
        board: Board,
        mut alpha: i64,
        beta: i64,
        depth: usize,
    ) -> (i64, Option<ChessMove>) {
        // https://www.chessprogramming.org/Alpha-Beta#Negamax_Framework
        if depth == 0 {
            return Evaluator::quiescence(board, alpha, beta);
        }

        let mut best_eval = i64::MIN + 1;
        let mut best_move = None;

        for m in Sorter::all(board) {
            let (mut new_score, _) =
                Searcher::alpha_beta(board.make_move_new(m), -beta, -alpha, depth - 1);
            new_score = -new_score;

            if new_score > best_eval {
                best_eval = new_score;
                best_move = Some(m);

                if new_score > alpha {
                    alpha = new_score;
                }
            }
            if new_score >= beta {
                return (best_eval, Some(m));
            }
        }

        (best_eval, best_move)
    }
}
