use chess::{File, Piece};

use crate::evaluator::{Evaluator, pawn::PAWN_VALUE};

pub(super) const KNIGHT_VALUE: i64 = 300;

impl Evaluator {
    pub(super) fn __knight_value(&self, value: &mut i64) {
        let total_pawns = (self.board.pieces(Piece::Pawn)).popcnt() as usize;
        // Decrease knight value for each missing pawn on the board.
        let mut penalty = [0; 17];
        for idx in 0..17 {
            penalty[idx] = (16 - idx) as i64 * (PAWN_VALUE / 10);
        }

        let own_knights = self.board.pieces(Piece::Knight) & self.board.color_combined(self.color);

        // Punish knights on the edge of the board.
        let own_edge_knights = own_knights
            .into_iter()
            .filter(|pos| pos.get_file() == File::A)
            .filter(|pos| pos.get_file() == File::H)
            .count() as i64;
        *value -= own_edge_knights * PAWN_VALUE;

        let count = own_knights.popcnt() as i64;
        *value += count * (KNIGHT_VALUE - penalty[total_pawns]);
    }

    pub(super) fn __knight_mobility(&self) -> i64 {
        let mut total_eval = 0;

        let knights = self.board.color_combined(self.color) & self.board.pieces(Piece::Knight);
        for knight in knights {
            let own_pieces = self.board.color_combined(self.color);
            let defended = chess::get_knight_moves(knight) & own_pieces;

            // TODO: maybe add knights mobility additionally to only defending?

            // For each protected piece, reward.
            total_eval += PAWN_VALUE * (defended.popcnt() as i64);
        }

        total_eval
    }
}
