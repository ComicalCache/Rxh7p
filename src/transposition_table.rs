use chess::{ChessMove, Color};
use fnv::FnvHashMap;

#[derive(Clone, Copy)]
pub enum TtEntryFlag {
    Exact,
    Beta,
    Alpha,
}

pub struct TranspositionTable {
    // Using a FnvHashMap should never overwrite any entries.
    // FIXME: when replacing for a more performant solution in the future, that fact needs to be
    // considered where ever entries are used!
    pub entries: FnvHashMap<u64, TtEntry>,
}

impl TranspositionTable {
    pub fn new() -> Self {
        TranspositionTable {
            entries: FnvHashMap::default(),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn get(&self, hash: u64) -> Option<&TtEntry> {
        self.entries.get(&hash)
    }

    pub fn set(&mut self, hash: u64, entry: TtEntry) {
        // Depth replacement.
        if let Some(curr_entry) = self.entries.get(&hash) {
            if curr_entry.depth <= entry.depth {
                self.entries.insert(hash, entry);
            }
        } else {
            self.entries.insert(hash, entry);
        }
    }
}

#[derive(Clone, Copy)]
pub struct TtEntry {
    /// Type of entry
    pub flag: TtEntryFlag,
    /// What depth was the entry recorded at
    pub depth: u16,
    /// The move
    pub mv: Option<ChessMove>,
    /// The color of the player making the move
    color: Color,
    /// Position value
    value: i64,
}

impl TtEntry {
    pub fn new(
        flag: TtEntryFlag,
        depth: u16,
        mv: Option<ChessMove>,
        color: Color,
        value: i64,
    ) -> Self {
        TtEntry {
            flag,
            depth,
            mv,
            color,
            value,
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

impl Default for TtEntry {
    fn default() -> Self {
        Self {
            flag: TtEntryFlag::Exact,
            depth: 0,
            mv: None,
            color: Color::White,
            value: i64::MIN + 1,
        }
    }
}
