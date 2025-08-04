use chess::{Board, BoardStatus, Color, Piece, Square};

use crate::evaluator::{
    piece_tables::piece_table_value,
    piece_values::{END_GAME_PIECE_VALUES, MID_GAME_PIECE_VALUES},
};

const MATE_VALUE: i64 = -10_000_000;

/// Returns the value that a type of piece adds to the game phase calculation.
fn game_phase_value(piece: Piece) -> u32 {
    match piece {
        Piece::Pawn | Piece::King => 0,
        Piece::Knight | Piece::Bishop => 1,
        Piece::Rook => 2,
        Piece::Queen => 4,
    }
}

/// Calculates the tapered evaluation.
fn tapered_eval(tween: u32, mid_game_eval: i64, end_game_eval: i64) -> i64 {
    let mid_game_phase = i64::from(tween.min(24));
    let end_game_phase = 24 - mid_game_phase;
    (mid_game_eval * mid_game_phase + end_game_eval * end_game_phase) / 24
}

/// Returns the phase of the game on a scale [0, 24] with 0 being end game and 24 early game.
pub fn game_phase(board: &Board) -> u32 {
    // Calculate game phase.
    let mut phase = 0;

    let phase_pieces = [Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen];
    for phase_piece in phase_pieces {
        phase += board.pieces(phase_piece).popcnt() * game_phase_value(phase_piece);
    }

    phase
}

/// Returns if the side to move is in a pawn end game.
pub fn pawn_end_game(board: &Board) -> bool {
    let color = board.side_to_move();
    let own_pieces = board.color_combined(color);
    let own_king_and_pawns = (board.pieces(Piece::King) | board.pieces(Piece::Pawn)) & own_pieces;

    own_king_and_pawns == *own_pieces
}

/// Returns the value of a piece.
pub fn piece_value(board: &Board, piece: Piece) -> i64 {
    // Get piece value.
    let mid_game_value = MID_GAME_PIECE_VALUES[piece];
    let end_game_value = END_GAME_PIECE_VALUES[piece];

    tapered_eval(game_phase(board), mid_game_value, end_game_value)
}

/// Returns the value of a type of piece on a square.
pub fn piece_square_value(board: &Board, color: Color, piece: Piece, square: Square) -> i64 {
    // Get piece value.
    let (mut mid_game_value, mut end_game_value) = piece_table_value(color, piece, square);
    mid_game_value += MID_GAME_PIECE_VALUES[piece];
    end_game_value += END_GAME_PIECE_VALUES[piece];

    tapered_eval(game_phase(board), mid_game_value, end_game_value)
}

/// Evaluates the current board.
pub fn evaluate(board: &Board) -> i64 {
    // Stalemate is neutral, being in checkmate is VERY bad since it means the player checking
    // is in checkmate.
    match board.status() {
        BoardStatus::Stalemate => return 0,
        BoardStatus::Checkmate => return MATE_VALUE,
        BoardStatus::Ongoing => {}
    }

    let pieces = [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ];

    let mut phase = 0;

    // Own value.
    let mut own_mid_game_value = 0;
    let mut own_end_game_value = 0;
    let color = board.side_to_move();
    for piece in pieces {
        for square in board.color_combined(color) & board.pieces(piece) {
            let (mid, end) = piece_table_value(color, piece, square);
            own_mid_game_value += mid + MID_GAME_PIECE_VALUES[piece];
            own_end_game_value += end + END_GAME_PIECE_VALUES[piece];

            // Increase tapered evalauation towards end game for each piece.
            phase += game_phase_value(piece);
        }
    }

    // Opponent value.
    let mut opponent_mid_game_value = 0;
    let mut opponent_end_game_value = 0;
    let color = !color;
    for piece in pieces {
        for square in board.color_combined(color) & board.pieces(piece) {
            let (mid, end) = piece_table_value(color, piece, square);
            opponent_mid_game_value += mid + MID_GAME_PIECE_VALUES[piece];
            opponent_end_game_value += end + END_GAME_PIECE_VALUES[piece];

            // Increase tapered evalauation towards end game for each piece.
            phase += game_phase_value(piece);
        }
    }

    // Evaluation of the phases by themselves.
    let mid_game_eval = own_mid_game_value - opponent_mid_game_value;
    let end_game_eval = own_end_game_value - opponent_end_game_value;

    tapered_eval(phase, mid_game_eval, end_game_eval)
}
