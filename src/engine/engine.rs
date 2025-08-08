#[cfg(feature = "logging")]
use std::{
    env,
    fs::{File, OpenOptions},
    io::{BufWriter, Write},
};

use std::sync::mpsc::Receiver;

use chess::Board;

#[cfg(feature = "logging")]
use crate::engine::search::SearchLog;

use crate::{engine::search::Search, tt::TT, uci::UciSearchStop};

/// The chess engine itself, it performs the search and data keeping.
pub struct Engine {
    /// Initial position.
    pub(super) initial_board: u64,
    /// Internal board.
    pub(super) board: Board,
    /// Transposition table.
    pub(super) tt: TT,

    /// Stack of played positions to detect threefold repetitions.
    pub(super) position_stack: Vec<(u64, bool)>,

    // TODO: 50 move rule.
    /// Information about the current search.
    pub(super) search: Search,

    /// Channel for receiving the search stop commands.
    pub(super) search_stop_rx: Receiver<UciSearchStop>,

    #[cfg(feature = "logging")]
    /// The log file to write the log to.
    pub(super) log_file: BufWriter<File>,

    #[cfg(feature = "logging")]
    /// Contains a log about the current search.
    pub search_log: SearchLog,
}

impl Engine {
    /// Creates a new engine.
    pub fn new(search_stop_rx: Receiver<UciSearchStop>) -> Self {
        #[cfg(feature = "logging")]
        let log_file = {
            let log_path = env::args()
                .nth(1)
                .expect("Expected log file path in logging build");
            let mut log_file = match OpenOptions::new().create(true).append(true).open(log_path) {
                Ok(file) => file,
                Err(err) => panic!("Failed to open log file: {err}"),
            };
            if let Err(err) = writeln!(
                &mut log_file,
                "=== START LOG ===\n{}",
                SearchLog::search_stats_header()
            ) {
                panic!("Failed to write to log file: {err}");
            }

            log_file
        };

        let board = Board::default();

        // Preallocate 120 plys to avoid many memory allocations early on.
        let mut position_stack = Vec::with_capacity(120);
        position_stack.push((board.get_hash(), false));

        Engine {
            initial_board: board.get_hash(),
            board,
            tt: TT::new(),
            position_stack,
            search: Search::default(),
            search_stop_rx,

            #[cfg(feature = "logging")]
            log_file: BufWriter::new(log_file),

            #[cfg(feature = "logging")]
            search_log: SearchLog::default(),
        }
    }

    #[cfg(feature = "logging")]
    pub fn flush_log_file(&mut self) {
        if let Err(err) = self.log_file.flush() {
            panic!("Failed to flush log file: {err}");
        }
    }
}
