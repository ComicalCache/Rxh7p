use chess::{Board, BoardStatus};

use crate::{
    evaluator::Evaluator,
    sorter::Sorter,
    transposition_table::{TTEntry, TTEntryFlag, TranspositionTable},
};

pub struct Engine {
    pub tt: TranspositionTable,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            tt: TranspositionTable::new(),
        }
    }

    pub fn negamax(
        &mut self,
        board: Board,
        mut a: i64,
        b: i64,
        depth: u16,
        start_depth: u16,
    ) -> i64 {
        let prev_a = a;
        let hash = board.get_hash();
        let side = board.side_to_move();

        // if viable TT entry exists return eval
        let tt_entry = self.tt.get(hash);
        if tt_entry.hash == hash && tt_entry.depth >= depth {
            let eval = tt_entry.eval(side);
            match tt_entry.flag {
                TTEntryFlag::Exact => return eval,
                TTEntryFlag::Beta if eval >= b => return eval,
                TTEntryFlag::Alpha if eval <= a => return eval,
                _ => {}
            }
        }

        // quiescence search to avoid event horizon
        if depth == 0 {
            let eval = self.quiescence(board, a, b);

            let tt_entry = TTEntry::new(TTEntryFlag::Exact, depth, None, side, eval, hash);
            self.tt.set(hash, tt_entry);

            return eval;
        }

        // checkmate or stalemate
        if board.status() != BoardStatus::Ongoing {
            let eval = Evaluator::evaluate(&board);

            let tt_entry = TTEntry::new(TTEntryFlag::Exact, depth, None, side, eval, hash);
            self.tt.set(hash, tt_entry);

            return eval;
        }

        // search all sorted moves doing alpha-beta pruning
        let mut max_eval = i64::MIN + 1;
        let mut best_mv = None;
        for mv in Sorter::all(&board) {
            // evaluate new position
            let new_eval = -self.negamax(board.make_move_new(mv), -b, -a, depth - 1, start_depth);

            if new_eval > max_eval {
                max_eval = new_eval;
                best_mv = Some(mv);
            }

            if new_eval > a {
                a = new_eval;
            }

            // cut-off, move was too good, opponent would not allow it
            if new_eval >= b {
                break;
            }
        }

        // save TT entry
        let flag = match (max_eval <= prev_a, max_eval >= b) {
            (true, _) => TTEntryFlag::Alpha,
            (_, true) => TTEntryFlag::Beta,
            _ => TTEntryFlag::Exact,
        };
        let tt_entry = TTEntry::new(flag, depth, best_mv, side, max_eval, hash);
        self.tt.set(hash, tt_entry);

        max_eval
    }

    fn quiescence(&self, board: Board, mut a: i64, b: i64) -> i64 {
        let mut max_eval = Evaluator::evaluate(&board);

        // cut-off, move was too good, opponent would not allow it
        if max_eval >= b {
            return max_eval;
        }
        if max_eval > a {
            a = max_eval;
        }

        for capture in Sorter::quiescence(&board) {
            // evaluate new position
            let new_eval = -self.quiescence(board.make_move_new(capture), -b, -a);

            if new_eval > max_eval {
                max_eval = new_eval;
            }

            if new_eval > a {
                a = new_eval;
            }

            // cut-off, move was too good, opponent would not allow it
            if new_eval >= b {
                break;
            }
        }

        max_eval
    }
}
