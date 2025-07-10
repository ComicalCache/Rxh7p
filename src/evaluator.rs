use chess::{Board, ChessMove, Color, Piece};

use crate::sorter::Sorter;

const PAWN_VALUE: i64 = 100;
const BISHOP_VALUE: i64 = 300;
const KNIGHT_VALUE: i64 = 300;
const ROOK_VALUE: i64 = 500;
const QUEEN_VALUE: i64 = 900;

pub struct Evaluator {}

impl Evaluator {
    #[inline(always)]
    fn piece_count(board: Board, piece: Piece, color: Color) -> i64 {
        (board.pieces(piece) & board.color_combined(color)).popcnt() as i64
    }

    pub fn evaluate(board: Board) -> i64 {
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

    pub fn quiescence(board: Board, mut alpha: i64, beta: i64) -> (i64, Option<ChessMove>) {
        // https://www.chessprogramming.org/Quiescence_Search
        let mut best_score = Evaluator::evaluate(board);
        let mut best_move = None;

        if best_score >= beta {
            return (best_score, None);
        }
        if best_score > alpha {
            alpha = best_score;
        }

        for capture in Sorter::captures(board) {
            let (mut new_score, _) =
                Evaluator::quiescence(board.make_move_new(capture), -beta, -alpha);
            new_score = -new_score;

            if new_score >= beta {
                return (new_score, Some(capture));
            }
            if new_score > best_score {
                best_score = new_score;
                best_move = Some(capture);
            }
            if new_score > alpha {
                alpha = new_score;
            }
        }

        (best_score, best_move)
    }
}
