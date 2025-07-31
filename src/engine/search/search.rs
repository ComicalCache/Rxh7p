use std::time::{Duration, SystemTime};

use chess::ChessMove;

/// Contains information about the currently conducted search.
pub struct Search {
    /// Ponder mode.
    pub ponder: bool,

    /// Moves to search.
    pub moves: Vec<ChessMove>,
    /// Stop infinite search.
    pub stop_infinite: bool,

    /// Ply of played moves during search.
    pub ply: usize,
    /// Count of nodes searched.
    pub nodes: u64,

    /// When the search started.
    pub start_time: SystemTime,

    /// Determines if the position is volatile.
    pub volatility: bool,

    /// Search no ply deeper than this.
    pub depth: Option<usize>,
    /// Search no more nodes than this.
    pub node_limit: Option<u64>,
    /// Hard think time limit.
    pub hard_move_time: Option<Duration>,
    /// Soft think time limit.
    pub soft_move_time: Option<Duration>,
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
            volatility: false,
            depth: None,
            node_limit: None,
            hard_move_time: None,
            soft_move_time: None,
        }
    }
}
