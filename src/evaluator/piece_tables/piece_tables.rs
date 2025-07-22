use std::ops::Index;

use chess::{Color, Piece, Rank, Square};

use crate::evaluator::piece_tables::directories::{
    END_GAME_TABLE_DIRECTORY, MID_GAME_TABLE_DIRECTORY,
};

/// Struct containing the value of a piece on a specific square.
pub(super) struct PieceTable {
    /// Values of a piece on a specific square.
    pub(super) values: [i64; 64],
}

impl Index<Square> for PieceTable {
    type Output = i64;

    fn index(&self, index: Square) -> &Self::Output {
        let file = index.get_file();
        let rank = index.get_rank();

        // Cannonical representation of File::A = 0, File::B = 1, ...
        let file_index = file.to_index();
        // Cannonical representation of Rank::One = 0, Rank::Two = 1, ...
        let rank_index = rank.to_index() << 3;

        &self.values[file_index + rank_index]
    }
}

/// Struct containing the value of a type of piece on a specific square. Naming scheme borrowed from
/// the page tables in x86.
pub(super) struct PieceTableDirectory {
    // Piece tables for a specific piece.
    pub(super) values: [PieceTable; 6],
}

impl Index<Piece> for PieceTableDirectory {
    type Output = PieceTable;

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

/// Returns the (mid game, end game) piece table values for a piece of a color on a square.
pub fn piece_table_value(color: Color, piece: Piece, square: Square) -> (i64, i64) {
    let square = match color {
        Color::Black => square,
        Color::White => {
            // Swap ranks for white color, since the tables are built to be usable by black
            // directly.
            let rank = match square.get_rank() {
                Rank::First => Rank::Eighth,
                Rank::Second => Rank::Seventh,
                Rank::Third => Rank::Sixth,
                Rank::Fourth => Rank::Fifth,
                Rank::Fifth => Rank::Fourth,
                Rank::Sixth => Rank::Third,
                Rank::Seventh => Rank::Second,
                Rank::Eighth => Rank::First,
            };

            Square::make_square(rank, square.get_file())
        }
    };

    (
        MID_GAME_TABLE_DIRECTORY[piece][square],
        END_GAME_TABLE_DIRECTORY[piece][square],
    )
}
