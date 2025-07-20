use chess::ChessMove;

/// An entry used for move ordering containing the move and it's associated value.
#[derive(PartialEq, Eq)]
pub(super) struct OrdererEntry {
    /// Value of the move (used for ordering).
    pub(super) value: i64,
    // The move.
    pub(super) mv: ChessMove,
}

impl OrdererEntry {
    /// Creates a new OrdererEntry.
    pub(super) fn new(value: i64, mv: ChessMove) -> Self {
        OrdererEntry { value, mv }
    }
}

impl PartialOrd for OrdererEntry {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.value.cmp(&other.value))
    }
}

impl Ord for OrdererEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}
