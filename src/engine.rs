use std::{
    cmp::max,
    sync::mpsc::Receiver,
    time::{Duration, SystemTime},
};

use chess::{Board, BoardStatus, ChessMove, Color, Piece};

use crate::{
    evaluator::Evaluator,
    sorter::Sorter,
    transposition_table::{TranspositionTable, TtEntry, TtEntryFlag},
    uci::{GoCommandConfig, Uci},
};

#[derive(Debug)]
struct Search {
    // Ponder mode.
    ponder: bool,

    // Moves to search.
    moves: Vec<ChessMove>,
    // Stop infinite search.
    stop_infinite: bool,

    // Ply of played moves during search.
    ply: u16,
    // Count of nodes searched.
    nodes: u64,

    // When the search started.
    start_time: SystemTime,

    // Search no ply deeper than this.
    depth: Option<u16>,
    // Search no more nodes than this.
    node_limit: Option<u64>,
    // Think time limit.
    move_time: Option<Duration>,

    // For following iterations, store the principal variation move to play.
    pv_move: Option<ChessMove>,
}

impl Search {
    fn new() -> Self {
        Search {
            ponder: false,
            moves: Vec::new(),
            stop_infinite: false,
            ply: 0,
            nodes: 0,
            start_time: SystemTime::UNIX_EPOCH,
            depth: None,
            node_limit: None,
            move_time: None,
            pv_move: None,
        }
    }
}

pub struct Engine {
    // Internal board.
    pub board: Board,
    // Transposition table.
    pub tt: TranspositionTable,

    // Stack of played positions to detect threefold repetitions.
    position_stack: Vec<(Board, bool)>,

    // TODO: 50 move rule.

    // Ply of played moves. Needed to access previously computed TT entries in next search.
    pub board_ply: u16,

    // Information about the current search.
    search: Search,

    // Stop receiver. Receives if a stop command was sent.
    stop_rx: Receiver<()>,
    // Ponder receiver. Receives if a ponderhit command was sent.
    ponderhit_rx: Receiver<()>,
}

impl Engine {
    pub fn new(stop_rx: Receiver<()>, ponderhit_rx: Receiver<()>) -> Self {
        let board = Board::default();

        Engine {
            board,
            tt: TranspositionTable::new(),
            position_stack: vec![(board, true)],
            board_ply: 0,
            search: Search::new(),
            stop_rx,
            ponderhit_rx,
        }
    }

    pub fn init(&mut self, board: Board) {
        self.board = board;
        self.tt.clear();
        self.position_stack = vec![(board, true)];
        self.board_ply = 0;
        self.search = Search::new();
    }

    pub fn uci_position(&mut self, mut board: Board, moves: Option<Vec<ChessMove>>) {
        // Initial position does not match engine.
        // Safe to unwrap since always one board exists (default by default...).
        if board.get_hash() != self.position_stack.first().unwrap().0.get_hash() {
            self.init(board);
        }

        // Nothing more to do.
        if moves.is_none() {
            return;
        }

        // Safe to unwrap as it was tested before.
        let moves = moves.unwrap();
        let moves_len = moves.len();

        // Apply all moves on the start board.
        for (idx, mv) in moves.into_iter().enumerate() {
            board = board.make_move_new(mv);

            // Plus one since position 0 contains start board.
            if let Some(position) = self.position_stack.get(idx + 1) {
                // Position matches history, no action needed.
                if board == position.0 {
                    continue;
                }

                // Sent position deviates starting here. Clear vector to add all new positions.
                self.position_stack.drain(idx + 1..);
            }

            // Add new moves to position stack. This implicitly handles the new latest moves even
            // for an identical starting position since the position stack doesn't include them.
            let irreversible = self.move_is_irreversible(&board, mv);
            self.position_stack.push((board, irreversible));
        }

        // Drain position stack if there are now less moves than previously known moves. No minus
        // one because position stack contains initial position.
        if moves_len < self.position_stack.len() - 1 {
            self.position_stack.drain(moves_len..);
        }
        // Set new board to self.
        self.board = board;
        // Set new ply. Minus one since initial position is on the stack.
        self.board_ply = (self.position_stack.len() - 1) as u16;
    }

    pub fn go(&mut self, config: GoCommandConfig) {
        self.go_prelude(config);
        self.iterative_deepening();
        self.go_epilogue();
    }

    fn go_prelude(&mut self, config: GoCommandConfig) {
        self.search = Search::new();

        // Reset stop_rx and ponderhit rx as they might cause the next search to short circuit.
        while self.stop_rx.try_recv().is_ok() {}
        while self.ponderhit_rx.try_recv().is_ok() {}

        // Set search move settings.
        self.search.moves = config.searchmoves;

        // Set time appropriately to player clocks.
        // FIXME: improve time management.
        match self.board.side_to_move() {
            Color::White => {
                if let Some(time) = config.wtime {
                    let inc = config.winc.unwrap_or(Duration::ZERO);
                    // Just divide remaining time by 25.
                    self.search.move_time = Some((time + inc).div_f64(25.));
                }
            }
            Color::Black => {
                if let Some(time) = config.btime {
                    let inc = config.binc.unwrap_or(Duration::ZERO);
                    // Just divide remaining time by 25.
                    self.search.move_time = Some((time + inc).div_f64(25.));
                }
            }
        }
        // Go movetime was set.
        if let Some(time) = config.move_time {
            if let Some(move_time) = self.search.move_time {
                // If move time is less than previously calculated time, use that.
                if time < move_time {
                    self.search.move_time = Some(time);
                }
            } else {
                // No time yet yet.
                self.search.move_time = Some(time);
            }
        }

        self.search.node_limit = config.nodes;
        self.search.depth = config.depth;
        self.search.ponder = config.ponder;
    }

    fn go_epilogue(&self) {
        // Make best found move.
        let new_board = self.board.make_move_new(
            self.search
                .pv_move
                .expect("Previous search failed to find single move."),
        );
        // Find moves to ponder on when playing best move.
        let ponder_moves = Sorter::all(
            // Safe to unwrap since previous iterative deepening search found a move.
            &new_board,
            0,
            &self.tt,
            // Plus one since a move was played above.
            self.board_ply + 1,
            self.search.ply,
            None,
        );

        let best_move = self
            .tt
            .get(self.board.get_hash() + self.board_ply as u64)
            .expect("Failed to fetch board from TT")
            .mv
            .expect("Board TT entry does not have a best move");

        Uci::best_move(best_move, Some(ponder_moves.into_iter().take(5).collect()));
    }

    fn iterative_deepening(&mut self) {
        // Reset principal variation move of previous iterative search.
        self.search.pv_move = None;

        let searchmoves = if self.search.moves.is_empty() {
            None
        } else {
            Some(self.search.moves.clone())
        };

        // Always set start time even if no go movetime command was sent.
        self.search.start_time = SystemTime::now();

        for depth in 1.. {
            // i64::MIN + 1 to avoid overflow when negating the value.
            if let Some(new_eval) = self.negamax(
                self.board,
                &searchmoves,
                i64::MIN + 1,
                i64::MAX,
                depth,
                self.search.pv_move,
            ) {
                // Store best move of previous iteration to search first in next iteration.
                if let Some(entry) = self.tt.get(self.board.get_hash() + self.board_ply as u64) {
                    self.search.pv_move = entry.mv;
                }

                Uci::search_info(
                    depth,
                    SystemTime::now()
                        .duration_since(self.search.start_time)
                        .unwrap(),
                    self.search.nodes,
                    // FIXME: gather pv should not be done here on the hot path?
                    self.get_pv(depth),
                    new_eval,
                );
            } else {
                // Search was cancelled.
                break;
            }
        }
    }

    fn get_pv(&self, depth: u16) -> Vec<ChessMove> {
        let mut pv = Vec::with_capacity(depth as usize);
        let mut temp_board = self.board;
        let mut idx = 0;
        while let Some(entry) = self
            .tt
            .get(temp_board.get_hash() + self.board_ply as u64 + idx)
            && let Some(mv) = entry.mv
            && idx < depth as u64
        {
            pv.push(mv);
            temp_board = temp_board.make_move_new(mv);

            idx += 1;
        }

        pv
    }

    fn stop_negamax(&mut self) -> bool {
        if self.ponderhit_rx.try_recv().is_ok() {
            self.search.ponder = false;
        }

        // Never stop in ponder mode.
        if self.search.ponder {
            return false;
        }

        // Depth limit.
        if let Some(depth) = self.search.depth
            && self.search.ply >= depth
        {
            return true;
        }

        // Node limit.
        if let Some(nodes) = self.search.node_limit
            && self.search.nodes >= nodes
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

        // Infinite search.
        if self.search.stop_infinite || self.stop_rx.try_recv().is_ok() {
            self.search.stop_infinite = true;
            return true;
        }

        false
    }

    fn negamax(
        &mut self,
        board: Board,
        searchmoves: &Option<Vec<ChessMove>>,
        mut alpha: i64,
        beta: i64,
        depth: u16,
        pv_move: Option<ChessMove>,
    ) -> Option<i64> {
        if self.stop_negamax() {
            return None;
        }

        // Count node as visited.
        self.search.nodes += 1;

        // Return score of 0 if position is a three-fold repetition.
        if self.threefold_repetition() {
            return Some(0);
        }

        // Quiescence search to avoid event horizon.
        if depth == 0 {
            return Some(Engine::quiescence(board, alpha, beta));
        }

        // Checkmate or stalemate.
        if board.status() != BoardStatus::Ongoing {
            return Some(Evaluator::evaluate(&board));
        }

        let prev_alpha = alpha;
        let hash = board.get_hash() + self.board_ply as u64 + self.search.ply as u64;
        let side = board.side_to_move();

        // If viable entry exists return evaluation.
        if let Some(entry) = self.tt.get(hash)
            && entry.depth >= depth
        {
            let eval = entry.eval(side);
            match entry.flag {
                TtEntryFlag::Exact => return Some(eval),
                TtEntryFlag::Beta if eval >= beta => return Some(eval),
                TtEntryFlag::Alpha if eval <= alpha => return Some(eval),
                _ => {}
            }
        }

        // Search all sorted moves doing alpha-beta pruning.
        let mut max_eval = i64::MIN + 1;
        let mut best_mv = None;

        let moves = if let Some(searchmoves) = searchmoves {
            searchmoves
        } else {
            &Sorter::all(
                &board,
                depth,
                &self.tt,
                self.board_ply,
                self.search.ply,
                pv_move,
            )
        };

        let mut move_number = 1;
        for mv in moves {
            if self.search.ply == 0 {
                Uci::curr_move_info(*mv, move_number);
                move_number += 1;
            }

            let new_board = board.make_move_new(*mv);

            // Add new position to and increment search ply.
            let irreversible = self.move_is_irreversible(&new_board, *mv);
            self.position_stack.push((new_board, irreversible));
            self.search.ply += 1;

            // Evaluate new position.
            let new_eval = self.negamax(new_board, &None, -beta, -alpha, depth - 1, None);

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
                // If negamax returns None, time was up, return up the chain.
                return None;
            }
        }

        // Store entry.
        let flag = match (max_eval <= prev_alpha, max_eval >= beta) {
            (true, _) => TtEntryFlag::Alpha,
            (_, true) => TtEntryFlag::Beta,
            _ => TtEntryFlag::Exact,
        };
        let tt_entry = TtEntry::new(flag, depth, best_mv, side, max_eval);
        self.tt.set(hash, tt_entry);

        Some(max_eval)
    }

    fn quiescence(board: Board, mut alpha: i64, beta: i64) -> i64 {
        let mut max_eval = Evaluator::evaluate(&board);

        // Cut-off, move was too good, opponent would not allow it.
        if max_eval >= beta {
            return max_eval;
        }

        alpha = max(max_eval, alpha);

        for capture in Sorter::quiescence(&board) {
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
