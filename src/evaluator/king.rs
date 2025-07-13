use crate::evaluator::{Evaluator, pawn::PAWN_VALUE};

// Minues one, otherwise in mate situations it can't differentiate between mate and no move found.
pub(super) const KING_VALUE: i64 = i64::MAX - 1;

impl Evaluator {
    pub(super) fn __king_mobility(&self) -> i64 {
        let mut eval = [0; 9];
        // Punish of king has 0 squares to move.
        eval[0] = -PAWN_VALUE / 2;
        // Reward if king has less than 3 squares to move.
        eval[1] = PAWN_VALUE;
        eval[2] = PAWN_VALUE;
        // Punish if king has more than 3 squares to move.
        for idx in 4..9 {
            eval[idx] = -PAWN_VALUE;
        }

        let king_moves = chess::get_king_moves(self.board.king_square(self.color));
        let blocking = self.board.color_combined(self.color);

        eval[(king_moves & blocking).popcnt() as usize]
    }
}
