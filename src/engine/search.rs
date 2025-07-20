use std::time::{Duration, SystemTime};

use chess::ChessMove;

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

    /// Search no ply deeper than this.
    pub(super) depth: Option<u16>,
    /// Search no more nodes than this.
    pub(super) node_limit: Option<u64>,
    /// Think time limit.
    pub(super) move_time: Option<Duration>,
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
            depth: None,
            node_limit: None,
            move_time: None,
        }
    }
}
