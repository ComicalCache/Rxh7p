#[cfg(feature = "logging")]
use std::io::Write;

use std::{cmp::max, time::SystemTime};

use chess::{BitBoard, Board, BoardStatus, ChessMove, EMPTY, Piece};

#[cfg(feature = "logging")]
use crate::engine::search::MoveTimeLimitKind;

use crate::{
    engine::Engine,
    evaluator::{self, piece_value},
    orderer::{self, masks},
    tt::{TtEntry, TtEntryFlag},
    uci::{UciSearchStop, sender},
};

impl Engine {
    /// Performs an iterative deepenign search on the internal state.
    pub(super) fn iterative_deepening(&mut self) {
        // Use predetermined moves for search if specified.
        let moves = if self.search.moves.is_empty() {
            None
        } else {
            Some(self.search.moves.clone())
        };

        // Search volatility threshold.
        let volatility_threshold = piece_value(&self.board, Piece::Pawn) / 2;

        // Store previous PV move to check for PV changes, or default is non is available.
        let mut prev_pv = self
            .tt
            .get(self.board.get_hash())
            .map_or(ChessMove::default(), |entry| entry.mv);

        // Always set start time even if no go movetime command was sent.
        self.search.start_time = SystemTime::now();

        let mut eval = 0;
        let mut pv_depth = 0;
        for depth in 1.. {
            // Use passed depth or at most depth 35. The PVS search stop must not check for depth
            // because of this. Search extensions will not be affected by this limit however.
            if !self.search.ponder && depth >= self.search.depth.unwrap_or(35) {
                break;
            }

            // Check if the next search should be started if time control is enabled.
            if !self.start_next_iteration() {
                #[cfg(feature = "logging")]
                {
                    self.search_log.skipped_next_iteration = true;
                    self.search_log.time_limit_kind = Some(MoveTimeLimitKind::Skip);
                }

                break;
            }

            let new_eval = self.pvs::<true>(self.board, moves.as_ref(), -i64::MAX, i64::MAX, depth);

            // If the search was not interrupted.
            if let Some(new_eval) = new_eval {
                // Safe to unwrap since if search finished the entry must exist.
                let new_pv = self.tt.get(self.board.get_hash()).unwrap().mv;

                // If eval changes a lot, extend search time.
                // If PV changes, extend search time.
                self.search.volatility =
                    (eval - new_eval).abs() > volatility_threshold || prev_pv != new_pv;

                // Set the PV search depth to the current depth and eval to new_eval.
                eval = new_eval;
                pv_depth = depth;
                prev_pv = new_pv;
            }

            // Send information of iteration but skip first four iterations to decrease traffic.
            // Don't skip if search is cancelled before the fith iteration.
            if depth > 4 || new_eval.is_none() {
                let search_time = SystemTime::now()
                    .duration_since(self.search.start_time)
                    .unwrap();

                // Only log to file if feature logging is enabled.
                #[cfg(feature = "logging")]
                {
                    let pv = self.pv(pv_depth);
                    sender::log_search_info(
                        &mut self.log_file,
                        depth,
                        search_time,
                        self.search.nodes,
                        pv,
                        eval,
                    );
                }

                // FIXME: gather PV should not be done here on the hot path?
                let pv = self.pv(pv_depth);
                sender::search_info(depth, search_time, self.search.nodes, pv, eval);
            }

            // Search was cancelled.
            if new_eval.is_none() {
                break;
            }
        }

        #[cfg(feature = "logging")]
        {
            self.search_log.depth = pv_depth;
            self.search_log.eval = eval;

            if let Err(err) = writeln!(&mut self.log_file, "{}", self.search_log) {
                panic!("Failed to write to log file: {err}");
            }

            // Only flush every 20 go commands to reduce overhead and have more comparable
            // performance.
            if self.search_log.ply % 20 == 0 {
                self.flush_log_file();
            }
        }
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

        // Only check this every couple of nodes to avoid getting the system time every ply.
        if self.search.nodes.trailing_zeros() >= 9 && self.stop_search_time() {
            return true;
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
    fn pvs<const PV: bool>(
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
        if self.reversible_repetitions() >= 3 {
            return Some(0);
        }

        // Checkmate or stalemate.
        let board_status = board.status();
        if board_status != BoardStatus::Ongoing {
            return Some(evaluator::evaluate(&board, Some(board_status)));
        }

        // Quiescence search to avoid event horizon at the end of search.
        if depth == 0 {
            return Some(Engine::quiescence(board, alpha, beta, None));
        }

        let tt_entry = self.tt.get(board.get_hash());
        // Skip lookup if position occured twice already to search a threefold repetition position.
        // If viable entry exists return evaluation.
        if self.reversible_repetitions() <= 1
            && let Some(entry) = tt_entry
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

        // Internal iterative deepening if no move exists yet.
        if PV && depth > 5 && tt_entry.is_none() {
            #[cfg(feature = "logging")]
            {
                self.search_log.iterative_deepening += 1;
            }

            // If PVS returns None, search was cancelled, return up the chain.
            self.pvs::<false>(board, searchmoves, alpha, beta, depth - 2)?;
        }

        // Null move pruning if not in check or in pawn end game.
        if !PV && depth >= 3 && !evaluator::pawn_end_game(&board)
            // Checks that not in check.
            && let Some(new_board) = board.null_move()
        {
            // Reduce depth by three in null move search and beta null window. Invert result due to
            // symmetry. If PVS returns None, search was cancelled, return up the chain.
            let eval = -self.pvs::<false>(new_board, searchmoves, -beta, -beta + 1, depth - 3)?;

            // Since null move failed high, best move will likely also fail high, prune.
            if eval >= beta {
                #[cfg(feature = "logging")]
                {
                    self.search_log.null_move_pruning += 1;
                }

                return Some(eval);
            }
        }

        // Use moves given by UCI or search all available (ordered) moves.
        let moves = if let Some(searchmoves) = searchmoves {
            searchmoves
        } else {
            &orderer::all(&board, depth, &self.tt)
        };

        let capture_mask = masks::captures_mask(&board);
        let futility_margin = if depth < 3 {
            // Piece value will never be big enough.
            #[allow(clippy::cast_possible_wrap)]
            let margin =
                piece_value(&board, Piece::Pawn) + piece_value(&board, Piece::Pawn) * depth as i64;

            // Can't be game over since no move was made and checked before.
            evaluator::evaluate(&board, None) + margin
        } else {
            // Could be anything but this guarantees alpha < alpha to always be false.
            alpha
        };

        let prev_alpha = alpha;
        let mut first_search = true;
        let mut max_eval = -i64::MAX;
        let mut best_move = None;
        for (idx, mv) in moves.iter().enumerate() {
            let new_board = board.make_move_new(*mv);

            let capture = capture_mask & BitBoard::from_square(mv.get_dest()) != EMPTY;
            let promotion = mv.get_promotion().is_some();
            let in_check = *board.checkers() != EMPTY;
            let check = in_check || *new_board.checkers() != EMPTY;

            // Futility pruning on quiet positions (not capture, check or promotion).
            if !PV && depth < 3 && futility_margin < alpha && !capture && !check && !promotion {
                #[cfg(feature = "logging")]
                {
                    self.search_log.futility_pruning += 1;
                }

                continue;
            }

            // Add new position to the stack and increment search ply.
            let irreversible = Engine::move_is_irreversible(&board, &new_board, *mv);
            self.position_stack
                .push((new_board.get_hash(), irreversible));
            self.search.ply += 1;

            let mut new_eval;
            if first_search {
                // Evaluate new position fully if first search. LMR will always be zero here thus
                // it's ommitted.
                new_eval = self.pvs::<PV>(new_board, None, -beta, -alpha, depth - 1);
                first_search = false;
            } else {
                // Calculate new reduction.
                let lmr = Engine::lmr(depth, idx, capture, promotion, in_check).min(depth - 1);

                // Perform null-window search on following searches with late move reduction.
                new_eval = self.pvs::<false>(new_board, None, -alpha - 1, -alpha, depth - 1 - lmr);

                // If the null-window search failed high, prune non-PV moves. In rare cases this
                // condition is true for PV moves, but the chance is negligible. Invert result due
                // to symmetry.
                if let Some(eval) = new_eval
                    && -eval > alpha
                    && -eval < beta
                {
                    #[cfg(feature = "logging")]
                    {
                        self.search_log.research_pvs += 1;
                    }

                    // Repeat the search with a full search without late move reduction.
                    new_eval = self.pvs::<true>(new_board, None, -beta, -alpha, depth - 1);
                }
            }

            // Pop new position from the stack and decrement search ply.
            self.position_stack.pop();
            self.search.ply -= 1;

            if let Some(new_eval) = new_eval {
                // Invert result due to symmetry.
                let new_eval = -new_eval;

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

        if let Some(mv) = best_move {
            let flag = match (max_eval <= prev_alpha, max_eval >= beta) {
                (true, _) => TtEntryFlag::Alpha,
                (_, true) => TtEntryFlag::Beta,
                _ => TtEntryFlag::Exact,
            };

            // Safe to unwrap, depth will never exceed 2^16...
            let depth = u16::try_from(depth).unwrap();
            let tt_entry = TtEntry::new(flag, depth, mv, board.side_to_move(), max_eval);
            self.tt.set(board.get_hash(), tt_entry);
        }

        Some(max_eval)
    }

    /// Performs a quiescence search on a given board.
    fn quiescence(
        board: Board,
        mut alpha: i64,
        beta: i64,
        board_status: Option<BoardStatus>,
    ) -> i64 {
        // First call can't be game over since no move was made and checked before.
        let mut max_eval = evaluator::evaluate(&board, board_status);

        // Cut-off, move was too good, opponent would not allow it.
        if max_eval >= beta {
            return max_eval;
        }

        // Delta pruning for hopeless positions.
        if max_eval + evaluator::piece_value(&board, Piece::Queen) < alpha {
            return alpha;
        }

        alpha = max(max_eval, alpha);

        for capture in orderer::quiescence(&board) {
            // Evaluate new position. Every move can lead to game over so it must be checked.
            let new_eval = -Engine::quiescence(
                board.make_move_new(capture),
                -beta,
                -alpha,
                Some(board.status()),
            );

            max_eval = max(new_eval, max_eval);
            alpha = max(new_eval, alpha);

            // Cut-off, move was too good, opponent would not allow it.
            if new_eval >= beta {
                break;
            }
        }

        max_eval
    }

    /// Calculates the late move depth reduction. Returns the (lmr, was set this call).
    fn lmr(depth: usize, move_number: usize, capture: bool, promotion: bool, check: bool) -> usize {
        // Don't reduce moves in high depth.
        // Don't reduce the first few (ordered) moves.
        // Don't reduce if in check.
        // Don't reduce captures.
        // Don't reduce promotions.
        if depth < 3 || move_number < 3 || check || capture || promotion {
            return 0;
        }

        // None of these are of concern.
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let reduction = ((depth as f32).ln() * ((move_number + 1) as f32).ln() / 2.5) as usize;
        reduction
    }
}
