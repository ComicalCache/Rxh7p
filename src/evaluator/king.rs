use chess::{BitBoard, Piece};

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

    pub(super) fn attack_enemy_king(&self) -> i64 {
        let enemy_king = self.board.king_square(!self.color);
        let squares_around = chess::get_king_moves(enemy_king);
        let target_squares = squares_around | BitBoard::from_square(enemy_king);

        let mut total_eval = 0;

        // Pawns.
        let pawns = self.board.color_combined(self.color) & self.board.pieces(Piece::Pawn);
        for pawn in pawns {
            let pawn_attacks = chess::get_pawn_attacks(pawn, self.color, target_squares);
            total_eval += pawn_attacks.popcnt() as i64 * (PAWN_VALUE / 5);
        }

        // Knights.
        let knights = self.board.color_combined(self.color) & self.board.pieces(Piece::Knight);
        for knight in knights {
            let knight_attacks = chess::get_knight_moves(knight) & target_squares;
            total_eval += knight_attacks.popcnt() as i64 * (PAWN_VALUE / 4);
        }

        // Bishops.
        let bishops = self.board.color_combined(self.color) & self.board.pieces(Piece::Bishop);
        for bishop in bishops {
            let bishop_attacks = chess::get_bishop_moves(bishop, target_squares);
            total_eval += bishop_attacks.popcnt() as i64 * (PAWN_VALUE / 3);
        }

        // Rooks.
        let rooks = self.board.color_combined(self.color) & self.board.pieces(Piece::Rook);
        for rook in rooks {
            let rook_attacks = chess::get_rook_moves(rook, target_squares);
            total_eval += rook_attacks.popcnt() as i64 * (PAWN_VALUE / 2);
        }

        // Queens.
        let queens = self.board.color_combined(self.color) & self.board.pieces(Piece::Queen);
        for queen in queens {
            let queen_attacks = chess::get_rook_moves(queen, target_squares)
                | chess::get_bishop_moves(queen, target_squares);
            total_eval += queen_attacks.popcnt() as i64 * (PAWN_VALUE / 2);
        }

        if self.board.color_combined(self.color).popcnt() > 14 {
            total_eval / 6
        } else if self.board.color_combined(self.color).popcnt() > 10 {
            total_eval / 3
        } else {
            total_eval
        }
    }
}
