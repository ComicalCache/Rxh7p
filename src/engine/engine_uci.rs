use std::time::Duration;

use chess::{Board, ChessMove, Color};

use crate::{
    engine::{Engine, search::Search},
    orderer,
    uci::{GoCommandConfig, UciSenderMessage},
};

impl Engine {
    /// Initializes the engine with defaults.
    pub fn init(&mut self, board: Board) {
        self.board = board;
        self.tt.clear();
        self.position_stack.clear();
        self.position_stack.push((board, true));
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

        // Initial position always exists. Remove all remaining elements of the vector.
        if self.position_stack.len() > 1 {
            self.position_stack.drain(1..);
        }

        // Apply all moves on the start board and add moves to the position stack.
        // Safe to unwrap as it was tested before.
        for mv in moves.unwrap() {
            let new_board = board.make_move_new(mv);

            let irreversible = Engine::move_is_irreversible(&board, &new_board, mv);
            self.position_stack.push((new_board, irreversible));

            board = new_board;
        }

        // Set board.
        self.board = board;
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

        // Set moves to search.
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
                // No time set yet.
                self.search.move_time = Some(time);
            }
        }

        // Set remaining parameters.
        self.search.node_limit = config.nodes;
        self.search.depth = config.depth;
        self.search.ponder = config.ponder;
    }

    /// Performs necessary tasks after a performed search.
    fn go_epilogue(&mut self) {
        // Find best move.
        let best_move = self
            .tt
            .get(self.board.get_hash())
            .expect("Failed to fetch board from TT")
            .mv;

        let new_board = self.board.make_move_new(best_move);

        // Find moves to ponder on after playing best move.
        let ponder_moves = orderer::all(&new_board, 0, &self.tt);
        // Give a selection of five moves to ponder on.
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
