use chess::{BitBoard, CastleRights, Color, EMPTY, File, Piece, Rank, Square};

use crate::evaluator::{Evaluator, pawn::PAWN_VALUE};

pub(super) const ROOK_VALUE: i64 = 500;

impl Evaluator {
    pub(super) fn __rook_value(&self, value: &mut i64) {
        let own_pawns = self.board.pieces(Piece::Pawn) & self.board.color_combined(self.color);
        let opponent_pawns =
            self.board.pieces(Piece::Pawn) & self.board.color_combined(!self.color);
        let total_pawns = (own_pawns.popcnt() + opponent_pawns.popcnt()) as usize;
        // Increase rook value for each missing pawn on the board.
        let mut reward = [0; 17];
        for idx in 0..17 {
            reward[idx] = (16 - idx) as i64 * (PAWN_VALUE / 10);
        }

        let own_rooks = self.board.pieces(Piece::Rook) & self.board.color_combined(self.color);

        for rook in own_rooks {
            let rook_file = chess::get_file(rook.get_file());
            let own_pawns_on_file = (own_pawns & rook_file).popcnt();
            let opponent_pawns_on_file = (opponent_pawns & rook_file).popcnt();

            // Reward rooks on (semi) open files.
            if own_pawns_on_file == 0 {
                if opponent_pawns_on_file == 0 {
                    *value += PAWN_VALUE;
                } else {
                    *value += PAWN_VALUE / 2;
                }
            }

            let opponent_queen =
                self.board.color_combined(!self.color) & self.board.pieces(Piece::Queen);
            let opponent_queen_on_file = (opponent_queen & rook_file).popcnt();

            // Reward rooks on same file as opponents queen.
            if opponent_queen_on_file != 0 {
                *value += PAWN_VALUE / 4;
            }
        }

        let count = own_rooks.popcnt() as i64;
        *value += count * (ROOK_VALUE + reward[total_pawns]);
    }

    pub(super) fn __rook_mobility(&self) -> i64 {
        let mut eval = [0; 15];
        // Punish rooks with less than three moving squares.
        for idx in 0..3 {
            eval[idx] = -PAWN_VALUE;
        }
        // Reward rooks with more than four moving squares.
        for idx in 5..15 {
            eval[idx] = PAWN_VALUE;
        }

        let mut total_eval = 0;

        let rooks = self.board.color_combined(self.color) & self.board.pieces(Piece::Rook);

        let unblocked = !self.board.combined();

        for rook in rooks {
            let rook_rays = chess::get_rook_rays(rook);
            let moves = ((rook_rays & unblocked) ^ rook_rays).popcnt() as usize;
            total_eval += eval[moves];
        }

        total_eval
    }

    pub(super) fn __rook_undefended(&self) -> i64 {
        let mut total_eval = 0;

        let rooks = self.board.color_combined(self.color) & self.board.pieces(Piece::Rook);

        let knight_attack_rays = Evaluator::knight_attack_rays(&self.board, self.color);
        let bishop_rays = Evaluator::bishop_rays(&self.board, self.color);
        let unblocked = !(self.board.combined() ^ rooks);

        for rook in rooks {
            let rook = BitBoard::from_square(rook);

            // Penalty if rook is not protected by bishops and/or knights.
            if rook & (knight_attack_rays & unblocked) == EMPTY
                && rook & (bishop_rays & unblocked) == EMPTY
            {
                total_eval = -PAWN_VALUE / 4;
            }
        }

        total_eval
    }

    pub(super) fn uncastled_block(&self) -> i64 {
        let mut total_eval = 0;

        let castle_rights = self.board.castle_rights(self.color);

        // Punish if uncastled with low rook mobility.
        if castle_rights == CastleRights::Both || castle_rights == CastleRights::KingSide {
            let blocking_second_h = Square::make_square(self.color.to_second_rank(), File::H);
            let blocking_third_h = match self.color {
                Color::Black => Square::make_square(Rank::Sixth, File::H),
                Color::White => Square::make_square(Rank::Third, File::H),
            };

            let g_piece = Square::make_square(self.color.to_my_backrank(), File::G);
            let f_piece = Square::make_square(self.color.to_my_backrank(), File::F);

            // Punish if can castle short and blocking rook by not castling.
            if self.board.piece_on(g_piece).is_none()
                && self.board.piece_on(f_piece).is_none()
                && (self.board.piece_on(blocking_second_h).is_some()
                    || self.board.piece_on(blocking_third_h).is_some())
            {
                total_eval -= 2 * PAWN_VALUE;
            }
        }

        total_eval
    }

    pub(super) fn connected_rooks(&self) -> i64 {
        let mut total_eval = 0;

        let rooks = self.board.color_combined(self.color) & self.board.pieces(Piece::Rook);

        for (r1, r2) in rooks.map_windows(|[r1, r2]| (*r1, *r2)) {
            let between = self.board.combined() & chess::between(r1, r2);

            // Reward connected rooks.
            if between == EMPTY {
                total_eval += PAWN_VALUE;
            }
        }

        total_eval
    }
}
