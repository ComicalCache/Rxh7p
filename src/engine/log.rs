use crate::{
    engine::{Engine, search::SearchLog},
    evaluator,
};

impl Engine {
    /// Initializes the log for the comming search.
    pub(super) fn go_log_prelude(&mut self) {
        self.search_log = SearchLog {
            hard_move_time: self.search.hard_move_time,
            soft_move_time: self.search.soft_move_time,
            time_limit_kind: None,
            skipped_next_iteration: false,
            ply: self.position_stack.len() - 1,
            game_phase: evaluator::game_phase(&self.board),
            research_pvs: 0,
            futility_pruning: 0,
            null_move_pruning: 0,
            eval: 0,
            depth: 0,
        };
    }
}
