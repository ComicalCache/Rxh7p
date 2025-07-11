use chess::{Board, Color, Piece};

const PAWN_VALUE: i64 = 100;
const BISHOP_VALUE: i64 = 350;
const KNIGHT_VALUE: i64 = 300;
const ROOK_VALUE: i64 = 500;
const QUEEN_VALUE: i64 = 900;
// minues one, otherwise in mate situations it can't differentiate between mate and no move found
const KING_VALUE: i64 = i64::MAX - 1;

pub struct Evaluator {}

impl Evaluator {
    pub fn piece_value(piece: Piece) -> i64 {
        match piece {
            Piece::Pawn => PAWN_VALUE,
            Piece::Knight => KNIGHT_VALUE,
            Piece::Bishop => BISHOP_VALUE,
            Piece::Rook => ROOK_VALUE,
            Piece::Queen => QUEEN_VALUE,
            Piece::King => KING_VALUE,
        }
    }

    pub fn evaluate(board: Board) -> i64 {
        // stalemate is neutral, being in checkmate is VERY bad
        match board.status() {
            chess::BoardStatus::Stalemate => return 0,
            chess::BoardStatus::Checkmate => return -KING_VALUE,
            _ => {}
        }

        // fetch count of opponents pieces
        let opponent_pawns = Evaluator::piece_count(board, Piece::Pawn, !board.side_to_move());
        let opponent_bishops = Evaluator::piece_count(board, Piece::Bishop, !board.side_to_move());
        let opponent_knights = Evaluator::piece_count(board, Piece::Knight, !board.side_to_move());
        let opponent_rooks = Evaluator::piece_count(board, Piece::Rook, !board.side_to_move());
        let opponent_queens = Evaluator::piece_count(board, Piece::Queen, !board.side_to_move());

        // sum total value of opponent pieces
        let opponent_value = opponent_pawns * PAWN_VALUE
            + opponent_bishops * BISHOP_VALUE
            + opponent_knights * KNIGHT_VALUE
            + opponent_rooks * ROOK_VALUE
            + opponent_queens * QUEEN_VALUE;

        // fetch count of own pieces
        let own_pawns = Evaluator::piece_count(board, Piece::Pawn, board.side_to_move());
        let own_bishops = Evaluator::piece_count(board, Piece::Bishop, board.side_to_move());
        let own_knights = Evaluator::piece_count(board, Piece::Knight, board.side_to_move());
        let own_rooks = Evaluator::piece_count(board, Piece::Rook, board.side_to_move());
        let own_queens = Evaluator::piece_count(board, Piece::Queen, board.side_to_move());

        // sum total value of own pieces
        let own_value = own_pawns * PAWN_VALUE
            + own_bishops * BISHOP_VALUE
            + own_knights * KNIGHT_VALUE
            + own_rooks * ROOK_VALUE
            + own_queens * QUEEN_VALUE;

        // difference between own and opponent piece value
        own_value - opponent_value
    }

    #[inline(always)]
    fn piece_count(board: Board, piece: Piece, color: Color) -> i64 {
        (board.pieces(piece) & board.color_combined(color)).popcnt() as i64
    }
}
