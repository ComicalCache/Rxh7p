use chess::{EMPTY, Piece};

use crate::evaluator::{Evaluator, pawn::PAWN_VALUE};

pub(super) const QUEEN_VALUE: i64 = 1000;

impl Evaluator {
    pub(super) fn __queen_value(&self, value: &mut i64) {
        let count = Evaluator::piece_count(&self.board, Piece::Queen, self.color);
        *value += count * QUEEN_VALUE;
    }

    pub(super) fn __queen_mobility(&self) -> i64 {
        let mut eval = [0; 27];
        // Punish queens with less than six moving squares.
        for idx in 0..6 {
            eval[idx] = -PAWN_VALUE;
        }
        // Reward queens with more than eight moving squares.
        for idx in 9..27 {
            eval[idx] = PAWN_VALUE;
        }

        let mut total_eval = 0;

        let queens = self.board.color_combined(self.color) & self.board.pieces(Piece::Queen);

        let unblocked = !self.board.combined();

        for queen in queens {
            let bishop_rays = chess::get_bishop_rays(queen);
            let rook_rays = chess::get_rook_rays(queen);
            let queen_rays = bishop_rays | rook_rays;

            let moves = ((queen_rays & unblocked) ^ queen_rays).popcnt() as usize;

            total_eval += eval[moves];
        }

        total_eval
    }

    pub(super) fn __queen_undefended(&self) -> i64 {
        let mut total_eval = 0;

        let queens = self.board.color_combined(self.color) & self.board.pieces(Piece::Queen);

        let knight_attack_rays = Evaluator::knight_attack_rays(&self.board, self.color);
        let bishop_rays = Evaluator::bishop_rays(&self.board, self.color);
        let rook_rays = Evaluator::rook_rays(&self.board, self.color);

        let unblocked = !(self.board.combined() ^ queens);

        // Penalty if queens are not protected by bishops, knights and/or rooks.
        if queens & (knight_attack_rays & unblocked) == EMPTY
            && queens & (bishop_rays & unblocked) == EMPTY
            && queens & (rook_rays & unblocked) == EMPTY
        {
            total_eval = -PAWN_VALUE / 4;
        }

        total_eval
    }
}
