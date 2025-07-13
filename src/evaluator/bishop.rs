use chess::Piece;

use crate::evaluator::{Evaluator, pawn::PAWN_VALUE};

pub(super) const BISHOP_VALUE: i64 = 350;

impl Evaluator {
    pub(super) fn __bishop_value(&self, value: &mut i64) {
        let count = Evaluator::piece_count(&self.board, Piece::Bishop, self.color);
        *value += count * BISHOP_VALUE;
    }

    pub(super) fn __bishop_mobility(&self) -> i64 {
        let mut eval = [0; 14];
        // Punish bishops with less than four moving squares.
        for idx in 0..4 {
            eval[idx] = -PAWN_VALUE;
        }
        // Reward bishops with more than four moving squares.
        for idx in 5..14 {
            eval[idx] = PAWN_VALUE;
        }

        let mut total_eval = 0;

        let bishops = self.board.color_combined(self.color) & self.board.pieces(Piece::Bishop);

        let unblocked = !self.board.combined();

        for bishop in bishops {
            let bishop_rays = chess::get_bishop_rays(bishop);
            let moves = ((bishop_rays & unblocked) ^ bishop_rays).popcnt() as usize;
            total_eval += eval[moves];
        }

        total_eval
    }

    pub(super) fn bishop_pair(&self) -> i64 {
        if (self.board.color_combined(self.color) & self.board.pieces(Piece::Bishop)).popcnt() == 2
        {
            return 0;
        }

        // Punish color weakness by losing one bishop.
        -PAWN_VALUE
    }
}
