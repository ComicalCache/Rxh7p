use chess::{ChessMove, Color};

/// Type of a TT entry.
#[derive(Clone, Copy)]
pub enum TtEntryFlag {
    /// Inside the search window.
    Exact,
    /// Beta cut-off.
    Beta,
    /// Lower bound.
    Alpha,
}

/// A TT entry contianing information about its type, found depth, move, color and value.
#[derive(Clone, Copy)]
pub struct TtEntry {
    /// Type of entry.
    pub flag: TtEntryFlag,
    /// What depth was the entry recorded at.
    pub depth: u16,
    /// The move.
    pub mv: ChessMove,
    /// The color of the player making the move.
    color: Color,
    /// Position value.
    value: i64,
}

impl TtEntry {
    /// Creates a new entry.
    pub fn new(flag: TtEntryFlag, depth: u16, mv: ChessMove, color: Color, value: i64) -> Self {
        TtEntry {
            flag,
            depth,
            mv,
            color,
            value,
        }
    }

    /// Returns the color adjusted value of the entry.
    pub fn value(&self, color: Color) -> i64 {
        // Adjust relative value to color that requests.
        if self.color == color {
            self.value
        } else {
            -self.value
        }
    }
}
