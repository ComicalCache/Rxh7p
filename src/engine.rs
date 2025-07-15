use std::{
    cmp::max,
    time::{Duration, SystemTime},
};

use chess::{Board, BoardStatus, ChessMove, Color, Piece};

use crate::{
    evaluator::Evaluator,
    printf,
    sorter::Sorter,
    transposition_table::{TTEntry, TTEntryFlag, TranspositionTable},
};

pub struct Engine {
    pub board: Board,
    pub tt: TranspositionTable,

    position_stack: Vec<(Board, bool)>,

    pub board_ply: u64,
    pub search_ply: u64,

    time_limit: Duration,
    start_time: SystemTime,

    prio_move: Option<ChessMove>,
}

impl Engine {
    pub fn new(board: Board) -> Self {
        Engine {
            board,
            tt: TranspositionTable::new(),
            position_stack: vec![(board, true)],
            board_ply: 0,
            search_ply: 0,
            time_limit: Duration::ZERO,
            start_time: SystemTime::UNIX_EPOCH,
            prio_move: None,
        }
    }

    pub fn make_move(&mut self, mv: ChessMove) {
        self.board = self.board.make_move_new(mv);

        let irreversible = self.move_is_irreversible(&self.board, mv);
        self.position_stack.push((self.board, irreversible));

        self.board_ply += 1;
    }

    pub fn iterative_deepening(&mut self, max_depth: u16, time_limit: Duration) -> (i64, u16) {
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
                self.negamax(self.board, i64::MIN + 1, i64::MAX, depth, self.prio_move)
            {
                eval = new_eval;

                // Store best move of previous iteration to search first in next iteration.
                if let Some(entry) = self.tt.get(self.board.get_hash() + self.board_ply) {
                    self.prio_move = entry.mv;
                }

                searched_depth = depth;

                // Make sure to overwrite everything previous.
                printf!("\r Depth: {depth} [EVAL={eval}]                   ");
            } else {
                // Search was cancelled.
                println!("\r Cancelled search at depth {depth} with evaluation: {eval}");
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

        // Return score of 0 if position is a three-fold repetition.
        if self.threefold_repetition() {
            return Some(0);
        }

        // Quiescence search to avoid event horizon.
        if depth == 0 {
            return Some(Engine::quiescence(board, a, b));
        }

        // Checkmate or stalemate.
        if board.status() != BoardStatus::Ongoing {
            return Some(Evaluator::evaluate(&board));
        }

        let prev_a = a;
        let hash = board.get_hash() + self.board_ply + self.search_ply;
        let side = board.side_to_move();

        // If viable entry exists return evaluation.
        if let Some(entry) = self.tt.get(hash)
            && entry.depth >= depth
        {
            let eval = entry.eval(side);
            match entry.flag {
                TTEntryFlag::Exact => return Some(eval),
                TTEntryFlag::Beta if eval >= b => return Some(eval),
                TTEntryFlag::Alpha if eval <= a => return Some(eval),
                _ => {}
            }
        }

        // Search all sorted moves doing alpha-beta pruning.
        let mut max_eval = i64::MIN + 1;
        let mut best_mv = None;

        for mv in Sorter::all(&board, self, depth, prio_move) {
            let new_board = board.make_move_new(mv);

            // Add new position to and increment search ply.
            let irreversible = self.move_is_irreversible(&new_board, mv);
            self.position_stack.push((new_board, irreversible));
            self.search_ply += 1;

            // Evaluate new position.
            let new_eval = self.negamax(new_board, -b, -a, depth - 1, None);

            // Pop new position from the stack and decrement search ply.
            self.position_stack.pop();
            self.search_ply -= 1;

            if let Some(mut new_eval) = new_eval {
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

    fn quiescence(board: Board, mut a: i64, b: i64) -> i64 {
        let mut max_eval = Evaluator::evaluate(&board);

        // Cut-off, move was too good, opponent would not allow it.
        if max_eval >= b {
            return max_eval;
        }

        a = max(max_eval, a);

        for capture in Sorter::quiescence(&board) {
            // Evaluate new position.
            let new_eval = -Engine::quiescence(board.make_move_new(capture), -b, -a);

            max_eval = max(new_eval, max_eval);
            a = max(new_eval, a);

            // Cut-off, move was too good, opponent would not allow it.
            if new_eval >= b {
                break;
            }
        }

        max_eval
    }

    fn threefold_repetition(&self) -> bool {
        // Can't be a three fold repetition if not sufficient moves have been played.
        if self.position_stack.len() < 8 {
            return false;
        }

        // Save to unwrap since at least 8 moves have been played.
        let target_hash = self.position_stack.last().unwrap().0.get_hash();
        let mut repetitions = 1;

        let len = self.position_stack.len();
        for idx in (0..len - 1).rev() {
            if self.position_stack[idx].1 {
                break;
            }

            // Check if it is a repetition.
            if self.position_stack[idx].0.get_hash() == target_hash {
                repetitions += 1;

                if repetitions == 3 {
                    return true;
                }
            }
        }

        false
    }

    fn move_is_irreversible(&self, board: &Board, mv: ChessMove) -> bool {
        // Save to unwrap since always at least one position exists after initialization.
        let before = self.position_stack.last().unwrap().0;

        // En-passant is lost forever.
        let en_passant = before.en_passant().is_some()
            || (before.en_passant().is_none() && board.en_passant().is_some());

        // Pawn moves are not reversible. This also encompases en-passant moves and promotions.
        let pawn_move = before.piece_on(mv.get_source()).unwrap() == Piece::Pawn;

        // Captures are not reversible.
        let capture = before.piece_on(mv.get_dest()).is_some();

        // Castling rights are not reversible.
        let castling_rights = (before.castle_rights(Color::Black)
            != board.castle_rights(Color::Black))
            || (before.castle_rights(Color::White) != board.castle_rights(Color::White));

        en_passant || pawn_move || capture || castling_rights
    }
}
