use std::{
    fmt::{Debug, Display},
    time::Duration,
};

/// Enum determining with what move time limit the search ended.
#[derive(Default)]
pub enum MoveTimeLimitKind {
    /// Soft move time limit.
    #[default]
    Soft,
    /// Hard move time limit.
    Hard,
}

impl Debug for MoveTimeLimitKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                MoveTimeLimitKind::Soft => "Soft",
                MoveTimeLimitKind::Hard => "Hard",
            }
        )
    }
}

/// Entry containing information about a search that can be logged.
#[derive(Default)]
pub struct SearchLog {
    /// How long the calculated hard move time for this search was.
    pub hard_move_time: Option<Duration>,
    /// How long the calculated soft move time for this search was.
    pub soft_move_time: Option<Duration>,
    /// If the search ended using the soft or hard move time limit.
    pub time_limit_kind: Option<MoveTimeLimitKind>,
    /// If the search stopped before starting a new iteration on low time.
    pub skipped_next_iteration: bool,

    /// The ply of the position.
    pub ply: usize,
    /// The game phase of the position.
    pub game_phase: u32,

    /// Evaluation of the search before checking for volatility.
    pub pre_volatility_eval: i64,
    /// Evaluation of the search.
    pub eval: i64,
    /// Search depth.
    pub depth: usize,
}

impl SearchLog {
    pub fn search_stats_header() -> &'static str {
        "[SEARCH STATS] ply,game phase,depth,skipped next iteration,pre volatility eval,eval,soft move time,hard move time,time limit kind"
    }
}

impl Display for SearchLog {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[SEARCH STATS] {},{},{},{},{},{},{:?},{:?},{:?}",
            self.ply,
            self.game_phase,
            self.depth,
            self.skipped_next_iteration,
            self.pre_volatility_eval,
            self.eval,
            self.soft_move_time,
            self.hard_move_time,
            self.time_limit_kind
        )
    }
}
