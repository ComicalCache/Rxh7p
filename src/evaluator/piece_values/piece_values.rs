use std::ops::Index;

use chess::Piece;

/// Struct containing the base value of a specific type of piece.
pub struct PieceValue {
    /// Base values for a specific type of piece.
    pub(super) values: [i64; 6],
}

impl Index<Piece> for PieceValue {
    type Output = i64;

    fn index(&self, index: Piece) -> &Self::Output {
        // Can't really assume cannonical order.
        match index {
            Piece::Pawn => &self.values[0],
            Piece::Knight => &self.values[1],
            Piece::Bishop => &self.values[2],
            Piece::Rook => &self.values[3],
            Piece::Queen => &self.values[4],
            Piece::King => &self.values[5],
        }
    }
}
