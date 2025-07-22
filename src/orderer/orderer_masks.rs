use chess::{BitBoard, Board, EMPTY};

/// Returns a mask for captures including en-passant moves.
pub(super) fn captures_mask(board: &Board) -> BitBoard {
    // Gets all captures.
    let captures = board.color_combined(!board.side_to_move());
    // En-passant moves are not included by the above mask since they don't land on the same
    // square of which they take.
    let en_passant = match board.en_passant() {
        Some(square) => BitBoard::from_square(square),
        None => EMPTY,
    };

    captures | en_passant
}
