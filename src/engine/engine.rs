use std::{cmp::max, sync::mpsc::Receiver, time::SystemTime};

use chess::{Board, BoardStatus, ChessMove, Piece};

#[cfg(feature = "logging")]
use crate::engine::search_log_entry::{MoveTimeLimitKind, SearchLogEntry};

use crate::{
    engine::search::Search,
    evaluator::{self, piece_value},
    orderer,
    tt::{TT, TtEntry, TtEntryFlag},
    uci::{UciSearchStop, uci_sender},
};

/// The chess engine itself, it performs the search and data keeping.
pub struct Engine {
    /// Initial position.
    pub(super) initial_board: u64,
    /// Internal board.
    pub(super) board: Board,
    /// Transposition table.
    pub(super) tt: TT,

    /// Stack of played positions to detect threefold repetitions.
    pub(super) position_stack: Vec<(u64, bool)>,

    // TODO: 50 move rule.
    /// Information about the current search.
    pub(super) search: Search,

    /// Channel for receiving the search stop commands.
    pub(super) search_stop_rx: Receiver<UciSearchStop>,

    #[cfg(feature = "logging")]
    /// Contains a log about the searches.
    pub search_log: Vec<SearchLogEntry>,
}

impl Engine {
    /// Creates a new engine.
    pub fn new(search_stop_rx: Receiver<UciSearchStop>) -> Self {
        let board = Board::default();

        // Preallocate 120 plys to avoid many memory allocations early on.
        let mut position_stack = Vec::with_capacity(120);
        position_stack.push((board.get_hash(), true));

        Engine {
            initial_board: board.get_hash(),
            board,
            tt: TT::new(),
            position_stack,
            search: Search::default(),
            search_stop_rx,

            #[cfg(feature = "logging")]
            // Preallocate 120 plys to avoid many memory allocations early on.
            search_log: Vec::with_capacity(120),
        }
    }

    /// Performs an iterative deepenign search on the internal state.
    pub(super) fn iterative_deepening(&mut self) {
        // Use predetermined moves for search if specified.
        let moves = if self.search.moves.is_empty() {
            None
        } else {
            Some(self.search.moves.clone())
        };

        // Always set start time even if no go movetime command was sent.
        self.search.start_time = SystemTime::now();

        // Search volatility threshold.
        let volatility_threshold = piece_value(&self.board, Piece::Pawn) / 2;

        let mut eval = 0;
        for depth in 1.. {
            // Use passed depth or at most depth 35. The PVS search stop must not check for depth
            // because of this. Search extensions will not be affected by this limit however.
            if !self.search.ponder && depth >= self.search.depth.unwrap_or(35) {
                break;
            }

            // i64::MIN + 1 to avoid overflow when negating the value.
            let new_eval = self.pvs(self.board, moves.as_ref(), i64::MIN + 1, i64::MAX, depth);

            // If the search was not interrupted.
            let mut pv_depth = depth - 1;
            if let Some(new_eval) = new_eval {
                // If eval changes a lot after 3rd ply, extend sort time.
                if depth > 3 && (eval - new_eval).abs() > volatility_threshold {
                    self.search.volatility = true;
                }

                // Set the PV search depth to the current depth and eval to new_eval.
                eval = new_eval;
                pv_depth = depth;
            }

            // Send information of iteration but skip first four iterations to decrease traffic.
            // Don't skip if search is cancelled before the fith iteration.
            if depth > 4 || new_eval.is_none() {
                let search_time = SystemTime::now()
                    .duration_since(self.search.start_time)
                    .unwrap();

                uci_sender::search_info(
                    depth,
                    search_time,
                    self.search.nodes,
                    // FIXME: gather PV should not be done here on the hot path?
                    self.pv(pv_depth),
                    eval,
                );
            }

            // Search was cancelled.
            if new_eval.is_none() {
                #[cfg(feature = "logging")]
                {
                    // Save to unwrap since log entry was added before.
                    self.search_log.last_mut().unwrap().depth = pv_depth;
                }

                break;
            }
        }
    }

    /// Returns the current principal variation of the internal state.
    fn pv(&mut self, depth: usize) -> Vec<ChessMove> {
        // At most print PV of eight plies.
        let depth = depth.min(8);
        let mut pv = Vec::with_capacity(depth);
        let mut temp_board = self.board;

        let mut idx = 0;
        // Traverse the TT until the searched depth and gather the PV.
        while let Some(mv) = self.tt.get(temp_board.get_hash()).map(|entry| entry.mv)
            && idx < depth
        {
            // Increment PV length at the beginning to be able to fully "unwind" the position stack.
            idx += 1;

            // Add new position to position stack to check for threefold repetition.
            let new_board = temp_board.make_move_new(mv);
            let irreversible = Engine::move_is_irreversible(&temp_board, &new_board, mv);
            self.position_stack
                .push((new_board.get_hash(), irreversible));
            // Only add the position to the PV if it is not a threefold repetition.
            if self.repetition() {
                break;
            }

            pv.push(mv);
            temp_board = new_board;
        }

        // Remove PV positions from the positions stack.
        for _ in 0..idx {
            self.position_stack.pop();
        }

        pv
    }

    /// Checks if a stop condition for search is fulfilled.
    fn stop_search(&mut self) -> bool {
        // Received ponderhit command.
        match self.search_stop_rx.try_recv() {
            Ok(UciSearchStop::Stop) => self.search.stop_infinite = true,
            Ok(UciSearchStop::Ponderhit) => self.search.ponder = false,
            _ => {}
        }

        // Stop command (infinite search, quit or ponder miss).
        if self.search.stop_infinite {
            return true;
        }

        // Never stop in ponder mode.
        if self.search.ponder {
            return false;
        }

        // Move time limit.
        if let Some(hard_time) = self.search.hard_move_time {
            // Safe to unwrap since always both are set.
            let soft_time = self.search.soft_move_time.unwrap();

            let duration = SystemTime::now()
                .duration_since(self.search.start_time)
                .unwrap();

            // Always stop when hard limit is reached.
            if duration > hard_time {
                #[cfg(feature = "logging")]
                {
                    // Save to unwrap since log entry was added before.
                    self.search_log.last_mut().unwrap().time_limit_kind =
                        Some(MoveTimeLimitKind::Hard);
                }

                return true;
            }

            // If the search was not volatile, abide to soft time limit.
            if !self.search.volatility && duration > soft_time {
                #[cfg(feature = "logging")]
                {
                    // Save to unwrap since log entry was added before.
                    self.search_log.last_mut().unwrap().time_limit_kind =
                        Some(MoveTimeLimitKind::Soft);
                }

                return true;
            }
        }

        // Node limit.
        if let Some(nodes) = self.search.node_limit
            && self.search.nodes > nodes
        {
            return true;
        }

        false
    }

    /// Performs a PVS search on a given board.
    fn pvs(
        &mut self,
        board: Board,
        searchmoves: Option<&Vec<ChessMove>>,
        mut alpha: i64,
        beta: i64,
        depth: usize,
    ) -> Option<i64> {
        if self.stop_search() {
            return None;
        }

        // Count node as visited.
        self.search.nodes += 1;

        // Return score of 0 if position is a three-fold repetition.
        if self.repetition() {
            return Some(0);
        }

        // Quiescence search to avoid event horizon.
        if depth == 0 {
            return Some(Engine::quiescence(board, alpha, beta));
        }

        // Checkmate or stalemate.
        if board.status() != BoardStatus::Ongoing {
            return Some(evaluator::evaluate(&board));
        }

        let prev_alpha = alpha;

        // If viable entry exists return evaluation.
        if let Some(entry) = self.tt.get(board.get_hash())
            && entry.depth as usize >= depth
        {
            let eval = entry.value(board.side_to_move());
            match entry.flag {
                TtEntryFlag::Exact => return Some(eval),
                TtEntryFlag::Beta if eval >= beta => return Some(eval),
                TtEntryFlag::Alpha if eval <= alpha => return Some(eval),
                _ => {}
            }
        }

        // Use moves given by UCI or search all available (ordered) moves.
        let moves = if let Some(searchmoves) = searchmoves {
            searchmoves
        } else {
            &orderer::all(&board, depth, &self.tt)
        };

        let mut first_search = true;
        let mut max_eval = i64::MIN + 1;
        let mut best_move = None;
        for mv in moves {
            let new_board = board.make_move_new(*mv);

            // Add new position to and increment search ply.
            let irreversible = Engine::move_is_irreversible(&board, &new_board, *mv);
            self.position_stack
                .push((new_board.get_hash(), irreversible));
            self.search.ply += 1;

            let mut new_eval;
            if first_search {
                // Evaluate new position fully if first search.
                new_eval = self.pvs(new_board, None, -beta, -alpha, depth - 1);
                first_search = false;
            } else {
                // Perform null-window search on following searches.
                new_eval = self.pvs(new_board, None, -alpha - 1, -alpha, depth - 1);

                // If the null-window search failed high, repeat with a full search.
                if let Some(eval) = new_eval
                // Prune non-PV moves. In rare cases this condition is true for PV moves, but the
                // chance is negligible. Inverse result due to symmetry.
                    && -eval > alpha
                    && -eval < beta
                {
                    new_eval = self.pvs(new_board, None, -beta, -alpha, depth - 1);
                }
            }

            // Pop new position from the stack and decrement search ply.
            self.position_stack.pop();
            self.search.ply -= 1;

            if let Some(mut new_eval) = new_eval {
                // Invert result due to symmetry.
                new_eval = -new_eval;

                if new_eval > max_eval {
                    max_eval = new_eval;
                    best_move = Some(*mv);
                }

                alpha = max(new_eval, alpha);

                // Cut-off, move was too good, opponent would not allow it.
                if new_eval >= beta {
                    break;
                }
            } else {
                // If PVS returns None, search was cancelled, return up the chain.
                return None;
            }
        }

        let best_move = best_move.expect("PVS didn't find any move to make.");

        // Store entry.
        let flag = match (max_eval <= prev_alpha, max_eval >= beta) {
            (true, _) => TtEntryFlag::Alpha,
            (_, true) => TtEntryFlag::Beta,
            _ => TtEntryFlag::Exact,
        };

        // Safe to unwrap, depth will never exceed 2^16...
        let depth = u16::try_from(depth).unwrap();
        let tt_entry = TtEntry::new(flag, depth, best_move, board.side_to_move(), max_eval);
        self.tt.set(board.get_hash(), tt_entry);

        Some(max_eval)
    }

    /// Performs a quiescence search on a given board.
    fn quiescence(board: Board, mut alpha: i64, beta: i64) -> i64 {
        let mut max_eval = evaluator::evaluate(&board);

        // Cut-off, move was too good, opponent would not allow it.
        if max_eval >= beta {
            return max_eval;
        }

        alpha = max(max_eval, alpha);

        for capture in orderer::quiescence(&board) {
            // Evaluate new position.
            let new_eval = -Engine::quiescence(board.make_move_new(capture), -beta, -alpha);

            max_eval = max(new_eval, max_eval);
            alpha = max(new_eval, alpha);

            // Cut-off, move was too good, opponent would not allow it.
            if new_eval >= beta {
                break;
            }
        }

        max_eval
    }
}
