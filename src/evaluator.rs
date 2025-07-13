mod bishop;
mod king;
mod knight;
mod pawn;
mod queen;
mod rook;
mod value;

use chess::{BitBoard, Board, Color, File, Piece};
use fnv::FnvHashMap;

use crate::evaluator::king::KING_VALUE;

pub struct Evaluator {
    stacked_pawns: [FnvHashMap<u64, PawnTableEntry>; 2],
    isolated_pawns: [FnvHashMap<u64, PawnTableEntry>; 2],
    board: Board,
    color: Color,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            stacked_pawns: [FnvHashMap::default(), FnvHashMap::default()],
            isolated_pawns: [FnvHashMap::default(), FnvHashMap::default()],
            board: Board::default(),
            color: Color::Black,
        }
    }

    pub fn evaluate(&mut self, board: Board) -> i64 {
        self.board = board;

        self.color = board.side_to_move();
        let own_eval = self.__evaluate();

        // self.color = !board.side_to_move();
        // let opponent_eval = self.__evaluate();

        own_eval // - opponent_eval
    }

    fn __evaluate(&mut self) -> i64 {
        // Stalemate is neutral, being in checkmate is VERY bad.
        match self.board.status() {
            chess::BoardStatus::Stalemate => return 0,
            chess::BoardStatus::Checkmate => return -KING_VALUE,
            _ => {}
        }

        self.value()
            + self.mobility()
            + self.stacked_pawns()
            + self.isolated_pawns()
            + self.blocked_center_pawns()
            + self.bishop_pair()
            + self.uncastled_block()
            + self.connected_rooks()
    }
}

impl Evaluator {
    fn piece_count(board: &Board, piece: Piece, color: Color) -> i64 {
        (board.color_combined(color) & board.pieces(piece)).popcnt() as i64
    }

    fn king_side(board: &Board, file: File, color: Color) -> bool {
        if board.king_square(color).get_file().to_index() <= 3 {
            file.to_index() <= 3
        } else {
            file.to_index() >= 4
        }
    }
}

struct PawnTableEntry {
    own_pawns: BitBoard,
    opponent_pawns: BitBoard,
    color: Color,
    eval: i64,
}

impl PawnTableEntry {
    fn new(own_pawns: BitBoard, opponent_pawns: BitBoard, color: Color, eval: i64) -> Self {
        PawnTableEntry {
            own_pawns,
            opponent_pawns,
            color,
            eval,
        }
    }

    fn collision(&self, own_pawns: BitBoard, opponent_pawns: BitBoard, color: Color) -> bool {
        if self.color == color {
            self.own_pawns != own_pawns || self.opponent_pawns != opponent_pawns
        } else {
            self.own_pawns != opponent_pawns || self.opponent_pawns != own_pawns
        }
    }
}
