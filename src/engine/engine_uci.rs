use chess::{Board, ChessMove};

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

    /// Returns the current principal variation of the internal state.
    pub(super) fn pv(&mut self, depth: usize) -> Vec<ChessMove> {
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
            if self.reversible_repetitions() >= 3 {
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
}
