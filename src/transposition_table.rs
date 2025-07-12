use chess::{ChessMove, Color};
use fnv::FnvHashMap;

#[derive(Clone, Copy)]
pub enum TTEntryFlag {
    Exact,
    Beta,
    Alpha,
}

pub struct TranspositionTable {
    pub entries: FnvHashMap<u64, TTEntry>,
    hits: u64,
    misses: u64,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            entries: FnvHashMap::default(),
            hits: 0,
            misses: 0,
        }
    }

    pub fn stats(&self) -> (u64, u64, u64) {
        (self.hits, self.misses, self.entries.len() as u64)
    }

    pub fn reset_stats(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }

    pub fn get(&mut self, hash: u64) -> Option<&TTEntry> {
        let res = self.entries.get(&hash);

        if let Some(_) = res {
            self.hits += 1;
        } else {
            self.misses += 1;
        }

        res
    }

    pub fn set(&mut self, hash: u64, entry: TTEntry) {
        self.entries.insert(hash, entry);
    }
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    /// Type of entry
    pub flag: TTEntryFlag,
    /// What depth was the entry recorded at
    pub depth: u16,
    /// The move,
    pub mv: Option<ChessMove>,
    /// The color of the player making the move
    color: Color,
    /// Position value
    value: i64,
}

impl TTEntry {
    pub fn new(
        flag: TTEntryFlag,
        depth: u16,
        mv: Option<ChessMove>,
        color: Color,
        value: i64,
    ) -> Self {
        TTEntry {
            flag,
            depth,
            mv,
            color,
            value,
        }
    }

    pub fn eval(&self, color: Color) -> i64 {
        // Adjust the relative value to tbe color that requests it.
        if self.color == color {
            self.value
        } else {
            -self.value
        }
    }
}

impl Default for TTEntry {
    fn default() -> Self {
        Self {
            flag: TTEntryFlag::Exact,
            depth: 0,
            mv: None,
            color: Color::White,
            value: i64::MIN + 1,
        }
    }
}
