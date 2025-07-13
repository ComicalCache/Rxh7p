use chess::{Color, File, Piece, Rank, Square};

use crate::evaluator::{Evaluator, PawnTableEntry, rook::ROOK_VALUE};

pub(super) const PAWN_VALUE: i64 = 100;

impl Evaluator {
    pub(super) fn __pawn_value(&self, value: &mut i64) {
        let count = Evaluator::piece_count(&self.board, Piece::Pawn, self.color);
        *value += count * PAWN_VALUE;
    }

    pub(super) fn __pawn_mobility(&self) -> i64 {
        let mut total_eval = 0;

        let pawns = self.board.color_combined(self.color) & self.board.pieces(Piece::Pawn);
        for pawn in pawns {
            let own_pieces = self.board.color_combined(self.color);

            let defended = chess::get_pawn_attacks(pawn, self.color, *own_pieces);

            // For each protected piece, reward.
            total_eval += (PAWN_VALUE / 2) * (defended.popcnt() as i64);
        }

        total_eval
    }

    pub(super) fn stacked_pawns(&mut self) -> i64 {
        use chess::File::*;

        let own_pawns = self.board.color_combined(self.color) & self.board.pieces(Piece::Pawn);
        let opponent_pawns =
            self.board.color_combined(!self.color) & self.board.pieces(Piece::Pawn);

        // Check if position is known and use cached evaluation.
        let hash = own_pawns & opponent_pawns;
        if let Some(entry) = self.stacked_pawns[self.color.to_index()].get(&hash.0)
            && !entry.collision(own_pawns, opponent_pawns, self.color)
        {
            return entry.eval;
        }

        let mut total_eval = 0;
        for file in [A, B, C, D, E, F, G, H] {
            let own_pawn_file = own_pawns & chess::get_file(file);
            let opponent_pawn_file = opponent_pawns & chess::get_file(file);

            let mut eval = 0;
            if own_pawn_file.popcnt() > 1 {
                // Punish stacked pawns.
                eval -= PAWN_VALUE / 2;

                // Increase punishment for blocked stacked pawns.
                if opponent_pawn_file.popcnt() != 0 {
                    eval *= 2;
                }

                // Decrease punishment for king side stacked pawns, defending the king with stacked
                // pawns should be punished less.
                if Evaluator::king_side(&self.board, file, self.color) {
                    eval /= 2;
                }
            }

            total_eval += eval;
        }

        // Store evaluation.
        self.stacked_pawns[self.color.to_index()].insert(
            hash.0,
            PawnTableEntry::new(own_pawns, opponent_pawns, self.color, total_eval),
        );

        total_eval
    }

    pub(super) fn isolated_pawns(&mut self) -> i64 {
        use chess::File::*;

        let own_pawns = self.board.color_combined(self.color) & self.board.pieces(Piece::Pawn);
        let opponent_pawns =
            self.board.color_combined(!self.color) & self.board.pieces(Piece::Pawn);

        // Check if position is known and use cached evaluation.
        let hash = own_pawns & opponent_pawns;
        if let Some(entry) = self.isolated_pawns[self.color.to_index()].get(&hash.0)
            && !entry.collision(own_pawns, opponent_pawns, self.color)
        {
            return entry.eval;
        }

        let mut total_eval = 0;

        // Punish isolated pawns in the middle more.
        let factors: [f64; 8] = [1., 1., 1.25, 1.5, 1.5, 1.25, 1., 1.];
        for (file, factor) in [A, B, C, D, E, F, G, H].into_iter().zip(factors) {
            let pawn_file = own_pawns & chess::get_file(file);
            let adjacent_pawns = own_pawns & chess::get_adjacent_files(file);

            let mut eval = 0;

            // Punish isolated pawns by a small factor.
            if pawn_file.popcnt() != 0 && adjacent_pawns.popcnt() == 0 {
                eval = -(((PAWN_VALUE / 2) as f64 * factor).round() as i64);
            }

            total_eval += eval;
        }

        // Store evaluation.
        self.isolated_pawns[self.color.to_index()].insert(
            hash.0,
            PawnTableEntry::new(own_pawns, opponent_pawns, self.color, total_eval),
        );

        total_eval
    }

    pub(super) fn blocked_center_pawns(&self) -> i64 {
        let square = |rank, file| Square::make_square(rank, file);

        let rank = self.color.to_second_rank();
        let third_rank = match self.color {
            Color::Black => Rank::Sixth,
            Color::White => Rank::Third,
        };

        let mut total_eval = 0;

        // Punish blocked center pawn.
        if let Some(Piece::Pawn) = self.board.piece_on(square(rank, File::D))
            && self.board.piece_on(square(third_rank, File::D)).is_some()
        {
            total_eval -= ROOK_VALUE;
        }

        // Punish blocked center pawn.
        if let Some(Piece::Pawn) = self.board.piece_on(square(rank, File::E))
            && self.board.piece_on(square(third_rank, File::E)).is_some()
        {
            total_eval -= ROOK_VALUE;
        }

        total_eval
    }
}
