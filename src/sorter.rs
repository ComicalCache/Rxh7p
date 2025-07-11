use chess::{BitBoard, Board, EMPTY, MoveGen};

pub struct Sorter {}

impl Sorter {
    pub fn all(board: Board) -> MoveGen {
        MoveGen::new_legal(&board)
    }

    pub fn captures(board: Board) -> MoveGen {
        let mut moves = MoveGen::new_legal(&board);

        let captures = board.color_combined(!board.side_to_move());
        // en passant moves are not included by the above mask since they don't land on the same
        // square of which they take
        let en_passant = match board.en_passant() {
            Some(square) => BitBoard::from_square(square),
            None => EMPTY,
        };
        moves.set_iterator_mask(*captures | en_passant);

        moves
    }
}
