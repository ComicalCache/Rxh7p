use std::{
    cmp::max,
    time::{Duration, SystemTime},
};

use chess::{Board, BoardStatus};

use crate::{
    evaluator::Evaluator,
    sorter::Sorter,
    transposition_table::{TTEntry, TTEntryFlag, TranspositionTable},
};

pub struct Engine {
    pub tt: TranspositionTable,

    time_limit: Duration,
    start_time: SystemTime,
    time_over: bool,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            tt: TranspositionTable::new(),
            time_limit: Duration::ZERO,
            start_time: SystemTime::UNIX_EPOCH,
            time_over: true,
        }
    }

    pub fn iterative_deepening(
        &mut self,
        board: Board,
        max_depth: u16,
        time_limit: Duration,
    ) -> i64 {
        let mut max_eval = 0;

        self.time_limit = time_limit;
        self.start_time = SystemTime::now();
        self.time_over = false;

        for depth in 1..=max_depth {
            // Search was cancelled, don't accept result.
            if self.time_over {
                break;
            }

            // i64::MIN + 1 to avoid overflow when negating the value.
            let new_eval = self.negamax(board, i64::MIN + 1, i64::MAX, depth);

            max_eval = max(new_eval, max_eval);

            println!("Evaluation at depth {depth}: {max_eval}");
        }

        max_eval
    }

    pub fn negamax(&mut self, board: Board, mut a: i64, b: i64, depth: u16) -> i64 {
        // Check if time over and cancel iterative search.
        if self.time_over
            || SystemTime::now().duration_since(self.start_time).unwrap() > self.time_limit
        {
            self.time_over = true;
            return 0;
        }

        let prev_a = a;
        let hash = board.get_hash();
        let side = board.side_to_move();

        // If viable entry exists return evaluation.
        let entry = self.tt.get(hash);
        if entry.hash == hash && entry.depth >= depth {
            let eval = entry.eval(side);
            match entry.flag {
                TTEntryFlag::Exact => return eval,
                TTEntryFlag::Beta if eval >= b => return eval,
                TTEntryFlag::Alpha if eval <= a => return eval,
                _ => {}
            }
        }

        // Quiescence search to avoid event horizon.
        if depth == 0 {
            let eval = self.quiescence(board, a, b);

            let tt_entry = TTEntry::new(TTEntryFlag::Exact, depth, None, side, eval, hash);
            self.tt.set(hash, tt_entry);

            return eval;
        }

        // Checkmate or stalemate.
        if board.status() != BoardStatus::Ongoing {
            let eval = Evaluator::evaluate(&board);

            let tt_entry = TTEntry::new(TTEntryFlag::Exact, depth, None, side, eval, hash);
            self.tt.set(hash, tt_entry);

            return eval;
        }

        // Search all sorted moves doing alpha-beta pruning.
        let mut max_eval = i64::MIN + 1;
        let mut best_mv = None;
        // FIXME: search best move from previous iteration.
        for mv in Sorter::all(&board) {
            // Evaluate new position.
            let new_eval = -self.negamax(board.make_move_new(mv), -b, -a, depth - 1);

            // Cancel iterative search.
            // Don't check time yourself since it's checked at start of previous recursive call and
            // the boolean flag is set accordingly.
            if self.time_over {
                return 0;
            }

            if new_eval > max_eval {
                max_eval = new_eval;
                best_mv = Some(mv);
            }

            a = max(new_eval, a);

            // Cut-off, move was too good, opponent would not allow it.
            if new_eval >= b {
                break;
            }
        }

        // Store entry.
        let flag = match (max_eval <= prev_a, max_eval >= b) {
            (true, _) => TTEntryFlag::Alpha,
            (_, true) => TTEntryFlag::Beta,
            _ => TTEntryFlag::Exact,
        };
        let tt_entry = TTEntry::new(flag, depth, best_mv, side, max_eval, hash);
        self.tt.set(hash, tt_entry);

        max_eval
    }

    fn quiescence(&mut self, board: Board, mut a: i64, b: i64) -> i64 {
        // Check if time over and cancel iterative search.
        if self.time_over
            || SystemTime::now().duration_since(self.start_time).unwrap() > self.time_limit
        {
            self.time_over = true;
            return 0;
        }

        let mut max_eval = Evaluator::evaluate(&board);

        // Cut-off, move was too good, opponent would not allow it.
        if max_eval >= b {
            return max_eval;
        }

        a = max(max_eval, a);

        for capture in Sorter::quiescence(&board) {
            // evaluate new position
            let new_eval = -self.quiescence(board.make_move_new(capture), -b, -a);

            // Cancel iterative search.
            // Don't check time yourself since it's checked at start of previous recursive call and
            // the boolean flag is set accordingly.
            if self.time_over {
                return 0;
            }

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
