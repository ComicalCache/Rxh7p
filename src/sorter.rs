use chess::{Board, ChessMove, MoveGen};

pub struct Sorter {}

impl Sorter {
    pub fn all(board: Board) -> Vec<ChessMove> {
        let moves = MoveGen::new_legal(&board);

        moves.collect()
    }

    pub fn captures(board: Board) -> Vec<ChessMove> {
        let moves = MoveGen::new_legal(&board);

        moves
            .filter(|m| board.piece_on(m.get_dest()).is_some())
            .collect()
    }
}
