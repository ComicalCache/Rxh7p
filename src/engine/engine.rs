use std::{
    cmp::max,
    sync::mpsc::{Receiver, Sender},
    time::SystemTime,
};

use chess::{Board, BoardStatus, ChessMove};

use crate::{
    engine::search::Search,
    evaluator, orderer,
    tt::{TT, TtEntry, TtEntryFlag},
    uci::UciSenderMessage,
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

    /// UCI sender. Sends messages to UCI on our behalf to avoid exepnsive stdio on hot path.
    pub(super) message_tx: Sender<UciSenderMessage>,
    /// Stop receiver. Receives if a stop command was sent.
    pub(super) stop_rx: Receiver<()>,
    /// Ponder receiver. Receives if a ponderhit command was sent.
    pub(super) ponderhit_rx: Receiver<()>,
}

impl Engine {
    /// Creates a new engine.
    pub fn new(
        message_tx: Sender<UciSenderMessage>,
        stop_rx: Receiver<()>,
        ponderhit_rx: Receiver<()>,
    ) -> Self {
        let board = Board::default();

        Engine {
            initial_board: board.get_hash(),
            board,
            tt: TT::new(),
            position_stack: vec![(board.get_hash(), true)],
            search: Search::default(),
            message_tx,
            stop_rx,
            ponderhit_rx,
        }
    }

    /// Performs an iterative deepenign search on the internal state.
    pub(super) fn iterative_deepening(&mut self) {
        // Use predetermined moves for search if specified.
        let searchmoves = if self.search.moves.is_empty() {
            None
        } else {
            Some(self.search.moves.clone())
        };

        // Always set start time even if no go movetime command was sent.
        self.search.start_time = SystemTime::now();

        let mut eval = 0;

        for depth in 1.. {
            // i64::MIN + 1 to avoid overflow when negating the value.
            let new_eval = self.pvs(self.board, &searchmoves, i64::MIN + 1, i64::MAX, depth);

            // Only set the PV search depth to the current depth and eval to new_eval if the search
            // was not interrupted.
            let mut pv_depth = depth - 1;
            if let Some(new_eval) = new_eval {
                eval = new_eval;
                pv_depth = depth;
            }

            // Send information of iteration but skip first four iterations to decrease traffic.
            // Don't skip if search is cancelled before the fith iteration.
            if depth > 4 || new_eval.is_none() {
                let search_time = SystemTime::now()
                    .duration_since(self.search.start_time)
                    .unwrap();

                self.message_tx
                    .send(UciSenderMessage::SearchInfo(
                        depth,
                        search_time,
                        self.search.nodes,
                        // FIXME: gather pv should not be done here on the hot path?
                        self.pv(pv_depth),
                        eval,
                    ))
                    .expect("Failed to send message to UCI sender.");
            }

            // Search was cancelled.
            if new_eval.is_none() {
                break;
            }
        }
    }

    /// Returns the current principal variation of the internal state.
    fn pv(&self, depth: u16) -> Vec<ChessMove> {
        let mut pv = Vec::with_capacity(depth as usize);
        let mut temp_board = self.board;

        let mut idx = 0;
        // Traverse the TT until the searched depth and gather the PV.
        while let Some(entry) = self.tt.get(temp_board.get_hash())
            && idx < depth as u64
        {
            pv.push(entry.mv);
            temp_board = temp_board.make_move_new(entry.mv);

            idx += 1;
        }

        pv
    }

    /// Checks if a stop condition for search is fulfilled.
    fn stop_search(&mut self) -> bool {
        // Received ponderhit command.
        if self.ponderhit_rx.try_recv().is_ok() {
            self.search.ponder = false;
        }

        // Received stop command.
        if self.stop_rx.try_recv().is_ok() {
            self.search.stop_infinite = true;
        }

        // Stop command (infinite search, quit or ponder miss).
        if self.search.stop_infinite {
            return true;
        }

        // Never stop in ponder mode.
        if self.search.ponder {
            return false;
        }

        // Depth limit.
        if let Some(depth) = self.search.depth
            && self.search.ply > depth
        {
            return true;
        }

        // Node limit.
        if let Some(nodes) = self.search.node_limit
            && self.search.nodes > nodes
        {
            return true;
        }

        // Move time limit.
        if let Some(move_time) = self.search.move_time
            && SystemTime::now()
                .duration_since(self.search.start_time)
                .unwrap()
                > move_time
        {
            return true;
        }

        false
    }

    /// Performs a PVS search on a given board.
    fn pvs(
        &mut self,
        board: Board,
        searchmoves: &Option<Vec<ChessMove>>,
        mut alpha: i64,
        beta: i64,
        depth: u16,
    ) -> Option<i64> {
        if self.stop_search() {
            return None;
        }

        // Count node as visited.
        self.search.nodes += 1;

        // Return score of 0 if position is a repetition.
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
        let side = board.side_to_move();

        // If viable entry exists return evaluation.
        if let Some(entry) = self.tt.get(board.get_hash())
            && entry.depth >= depth
        {
            let eval = entry.value(side);
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
        let mut best_mv = None;
        for mv in moves {
            let new_board = board.make_move_new(*mv);

            // Add new position to and increment search ply.
            let irreversible = Engine::move_is_irreversible(&board, &new_board, *mv);
            self.position_stack
                .push((new_board.get_hash(), irreversible));
            self.search.ply += 1;

            let mut new_eval;
            if !first_search {
                // Perform null-window search on following searches.
                new_eval = self.pvs(new_board, &None, -alpha - 1, -alpha, depth - 1);

                // If the null-window search failed high, repeat with a full search.
                if let Some(eval) = new_eval
                // Prune non-PV moves. In rare cases this condition is true for PV moves, but the
                // chance is negligible. Inverse result due to symmetry.
                    && -eval > alpha
                    && -eval < beta
                {
                    new_eval = self.pvs(new_board, &None, -beta, -alpha, depth - 1);
                }
            } else {
                // Evaluate new position fully if first search.
                new_eval = self.pvs(new_board, &None, -beta, -alpha, depth - 1);
                first_search = false;
            }

            // Pop new position from the stack and decrement search ply.
            self.position_stack.pop();
            self.search.ply -= 1;

            if let Some(mut new_eval) = new_eval {
                // Invert result due to symmetry.
                new_eval = -new_eval;

                if new_eval > max_eval {
                    max_eval = new_eval;
                    best_mv = Some(*mv);
                }

                alpha = max(new_eval, alpha);

                // Cut-off, move was too good, opponent would not allow it.
                if new_eval >= beta {
                    break;
                }
            } else {
                // If pvs returns None, search was cancelled, return up the chain.
                return None;
            }
        }

        // Store entry.
        let flag = match (max_eval <= prev_alpha, max_eval >= beta) {
            (true, _) => TtEntryFlag::Alpha,
            (_, true) => TtEntryFlag::Beta,
            _ => TtEntryFlag::Exact,
        };
        let tt_entry = TtEntry::new(
            flag,
            depth,
            best_mv.expect("PVS didn't find any move to make."),
            side,
            max_eval,
        );
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
