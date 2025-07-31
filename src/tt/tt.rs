use fnv::FnvHashMap;

use crate::tt::entry::TtEntry;

/// A transposition table storing previously evaluated moves.
pub struct TT {
    // Using a FnvHashMap should never overwrite any entries.
    // FIXME: when replacing for a more performant solution in the future, that fact needs to be
    // considered where ever entries are used!
    entries: FnvHashMap<u64, TtEntry>,
}

impl TT {
    /// Creates a new TT.
    pub fn new() -> Self {
        TT {
            entries: FnvHashMap::default(),
        }
    }

    /// Clears the TT (usefull when searching a new position without losing previously allocated
    /// memory).
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Returns a TT entry if it exists.
    pub fn get(&self, hash: u64) -> Option<&TtEntry> {
        self.entries.get(&hash)
    }

    /// Sets a TT entry. If the entry already exists it uses the depth replacement.
    pub fn set(&mut self, hash: u64, entry: TtEntry) {
        // Depth replacement.
        if let Some(curr_entry) = self.entries.get(&hash) {
            // Replace if new entry is from deeper search.
            // Replace if new entry is of higher priority and within depth distance.
            if curr_entry.depth <= entry.depth
                || (curr_entry.flag < entry.flag
                    && curr_entry.depth.abs_diff(entry.depth) < entry.flag.depth_distance())
            {
                self.entries.insert(hash, entry);
            }
        } else {
            self.entries.insert(hash, entry);
        }
    }
}
