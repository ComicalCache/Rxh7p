use std::{
    cmp::max,
    time::{Duration, SystemTime},
};

use chess::{Board, BoardStatus, ChessMove};

use crate::{
    evaluator::Evaluator,
    sorter::Sorter,
    transposition_table::{TTEntry, TTEntryFlag, TranspositionTable},
};

pub struct Engine {
    pub tt: TranspositionTable,

    time_limit: Duration,
    start_time: SystemTime,

    prio_move: Option<ChessMove>,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            tt: TranspositionTable::new(),
            time_limit: Duration::ZERO,
            start_time: SystemTime::UNIX_EPOCH,
            prio_move: None,
        }
    }

    pub fn iterative_deepening(
        &mut self,
        board: Board,
        max_depth: u16,
        time_limit: Duration,
    ) -> (i64, u16) {
        let mut eval = 0;

        // Set time constrains.
        self.time_limit = time_limit;
        self.start_time = SystemTime::now();

        // Reset prio move of previous iterative search.
        self.prio_move = None;

        let mut searched_depth = 0;
        for depth in 1..=max_depth {
            // i64::MIN + 1 to avoid overflow when negating the value.
            if let Some(new_eval) =
                self.negamax(board, i64::MIN + 1, i64::MAX, depth, self.prio_move)
            {
                eval = new_eval;

                // Store best move of previous iteration to search first in next iteration.
                if let Some(entry) = self.tt.get(board.get_hash()) {
                    self.prio_move = entry.mv;
                }

                searched_depth = depth;

                println!("Evaluation at depth {depth}: {eval}");
            } else {
                // Search was cancelled.
                println!("Cancelled search at depth {depth} with final: {eval}");
                break;
            }
        }

        (eval, searched_depth)
    }

    pub fn negamax(
        &mut self,
        board: Board,
        mut a: i64,
        b: i64,
        depth: u16,
        prio_move: Option<ChessMove>,
    ) -> Option<i64> {
        // Check if time over and cancel iterative search.
        if SystemTime::now().duration_since(self.start_time).unwrap() > self.time_limit {
            return None;
        }

        let prev_a = a;
        let hash = board.get_hash();
        let side = board.side_to_move();

        // If viable entry exists return evaluation.
        if let Some(entry) = self.tt.get(hash) {
            if entry.depth >= depth {
                let eval = entry.eval(side);
                match entry.flag {
                    TTEntryFlag::Exact => return Some(eval),
                    TTEntryFlag::Beta if eval >= b => return Some(eval),
                    TTEntryFlag::Alpha if eval <= a => return Some(eval),
                    _ => {}
                }
            }
        }

        // Quiescence search to avoid event horizon.
        if depth == 0 {
            return Some(self.quiescence(board, a, b));
        }

        // Checkmate or stalemate.
        if board.status() != BoardStatus::Ongoing {
            return Some(Evaluator::evaluate(&board));
        }

        // Search all sorted moves doing alpha-beta pruning.
        let mut max_eval = i64::MIN + 1;
        let mut best_mv = None;

        for mv in Sorter::all(&board, &self.tt, depth, prio_move) {
            // Evaluate new position.
            if let Some(mut new_eval) =
                self.negamax(board.make_move_new(mv), -b, -a, depth - 1, None)
            {
                // Invert result due to symmetry.
                new_eval = -new_eval;

                if new_eval > max_eval {
                    max_eval = new_eval;
                    best_mv = Some(mv);
                }

                a = max(new_eval, a);

                // Cut-off, move was too good, opponent would not allow it.
                if new_eval >= b {
                    break;
                }
            } else {
                // If negamax returns None, time was up, return up the chain.
                return None;
            }
        }

        // Store entry.
        let flag = match (max_eval <= prev_a, max_eval >= b) {
            (true, _) => TTEntryFlag::Alpha,
            (_, true) => TTEntryFlag::Beta,
            _ => TTEntryFlag::Exact,
        };
        let tt_entry = TTEntry::new(flag, depth, best_mv, side, max_eval);
        self.tt.set(hash, tt_entry);

        Some(max_eval)
    }

    fn quiescence(&mut self, board: Board, mut a: i64, b: i64) -> i64 {
        let mut max_eval = Evaluator::evaluate(&board);

        // Cut-off, move was too good, opponent would not allow it.
        if max_eval >= b {
            return max_eval;
        }

        a = max(max_eval, a);

        for capture in Sorter::quiescence(&board) {
            // evaluate new position
            let new_eval = -self.quiescence(board.make_move_new(capture), -b, -a);

            max_eval = max(new_eval, max_eval);
            a = max(new_eval, a);

            // Cut-off, move was too good, opponent would not allow it.
            if new_eval >= b {
                break;
            }
        }

        max_eval
    }
}
