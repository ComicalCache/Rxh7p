use std::time::Duration;

use chess::{Board, ChessMove, Color};

use crate::{
    engine::{Engine, Search},
    order::Orderer,
    uci::{GoCommandConfig, UciSenderMessage},
};

impl Engine {
    /// Initializes the engine with defaults.
    pub fn init(&mut self, board: Board) {
        self.board = board;
        self.tt.clear();
        self.position_stack = vec![(board, true)];
        self.board_ply = 0;
        self.search = Search::default();
    }

    /// Handles a received UCI position command and initializes itself accordingly.
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
            // Save to unwrap since always at least one position exists after initialization.
            let irreversible =
                Engine::move_is_irreversible(&self.position_stack.last().unwrap().0, &board, mv);
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

    /// Performes a received UCI go command search.
    pub fn go(&mut self, config: GoCommandConfig) {
        self.go_prelude(config);
        self.iterative_deepening();
        self.go_epilogue();
    }

    /// Performs necessary setup before starting the search.
    fn go_prelude(&mut self, config: GoCommandConfig) {
        self.search = Search::default();

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

    /// Performs necessary tasks after a performed search.
    fn go_epilogue(&mut self) {
        // Find best move.
        let best_move = self
            .tt
            .get(self.board.get_hash() + self.board_ply as u64)
            .expect("Failed to fetch board from TT")
            .mv;

        // If the predicted move was wrong and the ponder search was cancelled, skip playing the
        // move and adding to the position stack.
        if !(self.search.stop_infinite && self.search.ponder) {
            // Make best found move.
            let new_board = self.board.make_move_new(best_move);

            // Add move to position stack.
            let irreversible = Engine::move_is_irreversible(&self.board, &new_board, best_move);
            self.position_stack.push((new_board, irreversible));

            // Make move persistent.
            self.board = new_board;
        }

        // Find moves to ponder on when playing best move.
        let ponder_moves = Orderer::all(
            // Safe to unwrap since previous iterative deepening search found a move.
            &self.board,
            0,
            &self.tt,
            // Plus one since a move was played above.
            self.board_ply + 1,
            self.search.ply,
        );
        let ponder_moves = ponder_moves.into_iter().take(5).collect::<Vec<ChessMove>>();

        // Send reply over UCI.
        self.message_tx
            .send(UciSenderMessage::BestMove(
                best_move,
                if !ponder_moves.is_empty() {
                    Some(ponder_moves)
                } else {
                    None
                },
            ))
            .expect("Failed to send message to UCI sender.");
    }
}
