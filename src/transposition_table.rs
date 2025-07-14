use std::i64;

use chess::{ChessMove, Color};

pub const INDEX_BITS: usize = 25;
const NON_INDEX_BITS: usize = 64 - INDEX_BITS;
const MASK: usize = usize::MAX >> NON_INDEX_BITS;
const SIZE: usize = MASK + 1;

#[derive(Clone, Copy)]
pub enum TTEntryFlag {
    Exact,
    Beta,
    Alpha,
}

pub struct TranspositionTable {
    pub entries: Vec<TTEntry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            entries: vec![Default::default(); SIZE],
        }
    }

    pub fn get(&self, hash: u64) -> TTEntry {
        self.entries[hash as usize & MASK]
    }

    pub fn set(&mut self, hash: u64, entry: TTEntry) {
        let curr_entry = self.entries[hash as usize & MASK];

        // Depth replacement.
        if curr_entry.hash == hash && curr_entry.depth <= entry.depth {
            self.entries[hash as usize & MASK] = entry;
        }
    }
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    /// Type of entry
    pub flag: TTEntryFlag,
    /// What depth was the entry recorded at
    pub depth: u16,
    /// The move
    pub mv: Option<ChessMove>,
    /// The color of the player making the move
    color: Color,
    /// Position value
    value: i64,
    /// Full position has to detect colisions
    pub hash: u64,
}

impl TTEntry {
    pub fn new(
        flag: TTEntryFlag,
        depth: u16,
        mv: Option<ChessMove>,
        color: Color,
        value: i64,
        hash: u64,
    ) -> Self {
        TTEntry {
            flag,
            depth,
            mv,
            color,
            value,
            hash,
        }
    }

    pub fn eval(&self, color: Color) -> i64 {
        // Adjust relative value to color that requests.
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
            hash: 0,
        }
    }
}
