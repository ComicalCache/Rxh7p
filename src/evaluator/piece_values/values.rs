use crate::evaluator::piece_values::piece_values::PieceValue;

pub const MID_GAME_PIECE_VALUES: PieceValue = PieceValue {
    values: [
        82,   // Pawn.
        337,  // Knight.
        365,  // Bishop.
        477,  // Rook.
        1025, // Queen.
        0,    // King.
    ],
};

pub const END_GAME_PIECE_VALUES: PieceValue = PieceValue {
    values: [
        94,  // Pawn.
        281, // Knight.
        297, // Bishop.
        512, // Rook.
        936, // Queen.
        0,   // King.
    ],
};
