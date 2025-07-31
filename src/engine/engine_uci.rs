use std::time::Duration;

use chess::{Board, ChessMove, Color};

use crate::{
    engine::{Engine, search::Search},
    evaluator,
    uci::{GoCommandConfig, uci_sender},
};

impl Engine {
    /// Initializes the engine with defaults.
    pub fn uci_init(&mut self, board: Board) {
        self.initial_board = board.get_hash();
        self.board = board;
        self.tt.clear();
        self.position_stack.clear();
        self.position_stack.push((board.get_hash(), false));
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
        // Safe to unwrap as it was tested for none before.
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

        #[cfg(feature = "logging")]
        self.go_log_prelude();

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
        if let Some(best_move) = self.tt.get(self.board.get_hash()).map(|entry| entry.mv) {
            let new_board = self.board.make_move_new(best_move);

            // Send best follow-up as ponder move.
            uci_sender::best_move(
                best_move,
                self.tt.get(new_board.get_hash()).map(|entry| entry.mv),
            );
        }
    }

    fn set_move_time(&mut self, config: &GoCommandConfig) {
        // Decreases search time towards the beginning and end of the game and increases towards to
        // middle of the game. Game phase is calculated early game = 24 -> end game = 0.
        const MOVE_TIME_FACTORS: [(u64, u64); 25] = [
            (4, 5),  // 0.8
            (4, 5),  // 0.8
            (4, 5),  // 0.8
            (4, 5),  // 0.8
            (9, 10), // 0.9
            (9, 10), // 0.9
            (9, 10), // 0.9
            (9, 10), // 0.9
            (1, 1),  // 1
            (1, 1),  // 1
            (1, 1),  // 1
            (1, 1),  // 1
            (1, 1),  // 1
            (1, 1),  // 1
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (6, 5),  // 1.2
            (1, 1),  // 1
            (4, 5),  // 0.8
            (4, 5),  // 0.8
        ];

        if let Some((time, inc)) = match self.board.side_to_move() {
            Color::White => config
                .wtime
                .map(|time| (time, config.winc.unwrap_or(Duration::ZERO))),
            Color::Black => config
                .btime
                .map(|time| (time, config.binc.unwrap_or(Duration::ZERO))),
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
            let (num, denom) = MOVE_TIME_FACTORS[evaluator::game_phase(&self.board) as usize];

            // Scale time by game phase.
            #[allow(clippy::cast_possible_truncation)]
            let time = (num * time.as_millis() as u64) / denom;

            let safety_margin = 10;
            let soft_margin = 65;

            let hard_move_time = time - safety_margin;
            let soft_move_time = time - safety_margin - soft_margin;

            // Subtract a safety margin from the hard time limit to avoid losing by time.
            if time > safety_margin {
                self.search.hard_move_time = Some(Duration::from_millis(hard_move_time));
                self.search.soft_move_time = Some(Duration::from_millis(hard_move_time));

                // Set soft move time to be soft margin less than hard move time.
                if hard_move_time > soft_margin {
                    self.search.soft_move_time = Some(Duration::from_millis(soft_move_time));
                }
            }
        }
    }
}
