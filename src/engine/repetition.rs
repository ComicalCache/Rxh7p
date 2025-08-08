use chess::{Board, ChessMove, Color, Piece};

use crate::engine::Engine;

impl Engine {
    /// Counts the number of reversible repetitions of the last position on the position stack.
    pub(super) fn reversible_repetitions(&self) -> usize {
        // Save to unwrap since at least the start pos exists.
        let target_hash = self.position_stack.last().unwrap().0;

        let mut count = 0;
        for pos in self.position_stack.iter().rev() {
            count += usize::from(pos.0 == target_hash);

            // Iterate until the first irreversible move. Only check after possible increment to
            // find repetitions on the irreversible move.
            if pos.1 {
                break;
            }
        }

        count
    }

    /// Checks if a move is irreversible.
    pub(super) fn move_is_irreversible(
        board_before: &Board,
        board_after: &Board,
        mv: ChessMove,
    ) -> bool {
        // En-passant is lost forever.
        let en_passant = board_before.en_passant().is_some()
            || (board_before.en_passant().is_none() && board_after.en_passant().is_some());

        // Pawn moves are not reversible. This also encompases en-passant moves and promotions.
        // Save to unwrap since a piece must be standing on the source of the move.
        let pawn_move = board_before.piece_on(mv.get_source()).unwrap() == Piece::Pawn;

        // Captures are not reversible.
        let capture = board_before.piece_on(mv.get_dest()).is_some();

        // Castling rights are not reversible.
        let castling_rights = (board_before.castle_rights(Color::Black)
            != board_after.castle_rights(Color::Black))
            || (board_before.castle_rights(Color::White)
                != board_after.castle_rights(Color::White));

        en_passant || pawn_move || capture || castling_rights
    }
}
