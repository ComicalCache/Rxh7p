use crate::evaluator::piece_tables::{piece_tables::PieceTableDirectory, tables::*};

pub(super) const MID_GAME_TABLE_DIRECTORY: PieceTableDirectory = PieceTableDirectory {
    values: [
        PAWN_MID_GAME_TABLE,
        KNIGHT_MID_GAME_TABLE,
        BISHOP_MID_GAME_TABLE,
        ROOK_MID_GAME_TABLE,
        QUEEN_MID_GAME_TABLE,
        KING_MID_GAME_TABLE,
    ],
};

pub(super) const END_GAME_TABLE_DIRECTORY: PieceTableDirectory = PieceTableDirectory {
    values: [
        PAWN_END_GAME_TABLE,
        KNIGHT_END_GAME_TABLE,
        BISHOP_END_GAME_TABLE,
        ROOK_END_GAME_TABLE,
        QUEEN_END_GAME_TABLE,
        KING_END_GAME_TABLE,
    ],
};
