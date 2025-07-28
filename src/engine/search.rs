use std::time::{Duration, SystemTime};

use chess::ChessMove;

/// Threshold of pv volatility to cause increased search time.
pub const PV_VOLATILITY_THRESHOLD: u64 = 3;

/// Contains information about the currently conducted search.
pub(super) struct Search {
    /// Ponder mode.
    pub(super) ponder: bool,

    /// Moves to search.
    pub(super) moves: Vec<ChessMove>,
    /// Stop infinite search.
    pub(super) stop_infinite: bool,

    /// Ply of played moves during search.
    pub(super) ply: u16,
    /// Count of nodes searched.
    pub(super) nodes: u64,

    /// When the search started.
    pub(super) start_time: SystemTime,

    /// Count of how often VP changed.
    pub(super) pv_volatility: u64,

    /// Search no ply deeper than this.
    pub(super) depth: Option<u16>,
    /// Search no more nodes than this.
    pub(super) node_limit: Option<u64>,
    /// Hard think time limit.
    pub(super) hard_move_time: Option<Duration>,
    /// Soft think time limit.
    pub(super) soft_move_time: Option<Duration>,
}

impl Default for Search {
    fn default() -> Self {
        Search {
            ponder: false,
            moves: Vec::new(),
            stop_infinite: false,
            ply: 0,
            nodes: 0,
            start_time: SystemTime::UNIX_EPOCH,
            pv_volatility: 0,
            depth: None,
            node_limit: None,
            hard_move_time: None,
            soft_move_time: None,
        }
    }
}
