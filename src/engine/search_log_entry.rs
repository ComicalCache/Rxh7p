use std::{
    fmt::{Debug, Display},
    time::Duration,
};

/// Enum determining with what move time limit the search ended.
pub enum MoveTimeLimitKind {
    /// Soft move time limit.
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
pub struct SearchLogEntry {
    /// How long the calculated hard move time for this search was.
    pub(super) hard_move_time: Option<Duration>,
    /// How long the calculated soft move time for this search was.
    pub(super) soft_move_time: Option<Duration>,
    /// If the search ended using the soft or hard move time limit.
    pub(super) time_limit_kind: Option<MoveTimeLimitKind>,

    /// The ply of the position.
    pub(super) ply: usize,
    /// The game phase of the position.
    pub(super) game_phase: u32,
}

impl Display for SearchLogEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{},{},{:?},{:?},{:?}",
            self.ply,
            self.game_phase,
            self.soft_move_time,
            self.hard_move_time,
            self.time_limit_kind
        )
    }
}
