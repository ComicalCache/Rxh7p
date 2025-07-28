use std::time::Duration;

use chess::{Board, ChessMove, Color};

use crate::{
    engine::{Engine, search::Search},
    uci::{GoCommandConfig, uci_sender},
};

impl Engine {
    /// Initializes the engine with defaults.
    pub fn uci_init(&mut self, board: Board) {
        self.initial_board = board.get_hash();
        self.board = board;
        self.tt.clear();
        self.position_stack.clear();
        self.position_stack.push((board.get_hash(), true));
        self.search = Search::default();
    }

    /// Handles a received UCI position command and initializes itself accordingly.
    pub fn uci_position(&mut self, mut board: Board, moves: Option<Vec<ChessMove>>) {
        // Initial position does not match engine.
        if board.get_hash() != self.initial_board {
            self.uci_init(board);
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
            self.position_stack
                .push((new_board.get_hash(), irreversible));

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

        // Reset search_stop_rx as it might cause the next search to short circuit.
        while self.search_stop_rx.try_recv().is_ok() {}

        self.set_move_time(&config);

        // Set moves to search.
        self.search.moves = config.searchmoves;

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

        // Send best follow-up as ponder move.
        uci_sender::best_move(
            best_move,
            self.tt.get(new_board.get_hash()).map(|entry| entry.mv),
        );
    }

    fn set_move_time(&mut self, config: &GoCommandConfig) {
        if let Some((time, inc)) = match self.board.side_to_move() {
            Color::White => {
                if let Some(time) = config.wtime {
                    Some((time, config.winc.unwrap_or(Duration::ZERO)))
                } else {
                    None
                }
            }
            Color::Black => {
                if let Some(time) = config.btime {
                    Some((time, config.binc.unwrap_or(Duration::ZERO)))
                } else {
                    None
                }
            }
        } {
            // Just divide remaining time by 20 plus 3/4th of the increment.
            self.search.hard_move_time = Some((time / 20) + (3 * inc / 4));
        }

        // Go movetime was set.
        if let Some(time) = config.move_time {
            if let Some(move_time) = self.search.hard_move_time {
                // If move time is less than previously calculated time, use that.
                if time < move_time {
                    self.search.hard_move_time = Some(time);
                }
            } else {
                // No time set yet.
                self.search.hard_move_time = Some(time);
            }
        }

        // Set soft move time.
        self.search.soft_move_time = self.search.hard_move_time;

        if let Some(time) = self.search.hard_move_time {
            let ten_ms = Duration::from_millis(10);
            let fifty_ms = Duration::from_millis(50);

            // Subtract 10ms from the hard time limit to avoid losing by time.
            if time > ten_ms {
                self.search.hard_move_time = Some(time - ten_ms);
                self.search.soft_move_time = Some(time - ten_ms);

                // Set soft move time to be 50ms less than hard move time.
                if self.search.hard_move_time.unwrap() > fifty_ms {
                    self.search.soft_move_time = Some(time - ten_ms - fifty_ms);
                }
            }
        }
    }
}
