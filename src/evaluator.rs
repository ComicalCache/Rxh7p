use chess::{BitBoard, Board, Color, File, Piece};
use fnv::FnvHashMap;

const PAWN_VALUE: i64 = 100;
const BISHOP_VALUE: i64 = 350;
const KNIGHT_VALUE: i64 = 300;
const ROOK_VALUE: i64 = 500;
const QUEEN_VALUE: i64 = 1000;
// Minues one, otherwise in mate situations it can't differentiate between mate and no move found.
const KING_VALUE: i64 = i64::MAX - 1;

pub struct Evaluator {
    doubled_pawns: FnvHashMap<u64, PawnTableEntry>,
    isolated_pawns: FnvHashMap<u64, PawnTableEntry>,
}

impl Evaluator {
    pub fn new() -> Self {
        Evaluator {
            doubled_pawns: FnvHashMap::default(),
            isolated_pawns: FnvHashMap::default(),
        }
    }

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

    pub fn evaluate(&mut self, board: Board) -> i64 {
        // Stalemate is neutral, being in checkmate is VERY bad.
        match board.status() {
            chess::BoardStatus::Stalemate => return 0,
            chess::BoardStatus::Checkmate => return -KING_VALUE,
            _ => {}
        }

        self.value_difference(board) + self.doubled_pawns(board) + self.isolated_pawns(board)
    }

    fn value_difference(&self, board: Board) -> i64 {
        use chess::Piece::*;

        let mut total_opponent_pieces = 0;
        let mut opponent_value = 0;
        for piece in [Pawn, Bishop, Knight, Rook, Queen] {
            let piece_count = Evaluator::piece_count(board, piece, !board.side_to_move());
            total_opponent_pieces += piece_count;
            opponent_value += piece_count * Evaluator::piece_value(piece);
        }

        let mut total_own_pieces = 0;
        let mut own_value = 0;
        for piece in [Pawn, Bishop, Knight, Rook, Queen] {
            let piece_count = Evaluator::piece_count(board, piece, board.side_to_move());
            total_own_pieces += piece_count;
            own_value += piece_count * Evaluator::piece_value(piece);
        }

        // Difference between own and opponent piece value. Avoid dividing by zero.
        match (total_own_pieces, total_opponent_pieces) {
            (0, 0) => 0,
            (0, _) => -(opponent_value / total_opponent_pieces),
            (_, 0) => own_value / total_own_pieces,
            (_, _) => (own_value / total_own_pieces) - (opponent_value / total_opponent_pieces),
        }
    }

    fn doubled_pawns(&mut self, board: Board) -> i64 {
        use chess::File::*;

        let own_pawns = board.color_combined(board.side_to_move()) & board.pieces(Piece::Pawn);
        let opponent_pawns =
            board.color_combined(!board.side_to_move()) & board.pieces(Piece::Pawn);

        // Check if position is known and use cached evaluation.
        let hash = own_pawns & opponent_pawns;
        if let Some(entry) = self.doubled_pawns.get(&hash.0)
            && !entry.collision(own_pawns, opponent_pawns, board.side_to_move())
        {
            return entry.eval;
        }

        let mut total_eval = 0;
        for file in [A, B, C, D, E, F, G, H] {
            let own_pawn_file = own_pawns & chess::get_file(file);
            let opponent_pawn_file = opponent_pawns & chess::get_file(file);

            let mut eval = 0;
            if own_pawn_file.popcnt() > 1 {
                // Reduce half of a pawn value for doubled pawns.
                eval = -(PAWN_VALUE / 2);

                // If opponent has pawn on same file blocking our doubled pawns, reduct whole pawn
                // since multiple pawns are blocked.
                if opponent_pawn_file.popcnt() != 0 {
                    eval *= 2;
                }

                // Ff double pawns on king side half the penalty,defending the king with doubled
                // pawns should be punished less.
                if Evaluator::king_side(board, file) {
                    eval /= 2;
                }
            }

            // Flat reward fourth of a pawn for each doubled enemy pawn.
            // Idea behind asymmetric evaluation is that it incentivises against having doubled
            // pawns but doesn't incentivice causing doubled pawns too much.
            if opponent_pawn_file.popcnt() > 1 {
                eval += PAWN_VALUE / 4;
            }

            total_eval += eval;
        }

        // Store evaluation.
        self.doubled_pawns.insert(
            hash.0,
            PawnTableEntry::new(own_pawns, opponent_pawns, board.side_to_move(), total_eval),
        );

        total_eval
    }

    fn isolated_pawns(&mut self, board: Board) -> i64 {
        use chess::File::*;

        let own_pawns = board.color_combined(board.side_to_move()) & board.pieces(Piece::Pawn);
        let opponent_pawns =
            board.color_combined(!board.side_to_move()) & board.pieces(Piece::Pawn);

        // Check if position is known and use cached evaluation.
        let hash = own_pawns & opponent_pawns;
        if let Some(entry) = self.isolated_pawns.get(&hash.0)
            && !entry.collision(own_pawns, opponent_pawns, board.side_to_move())
        {
            return entry.eval;
        }

        let mut total_eval = 0;

        // Punish/reward isolated pawns in the middle more.
        let factors: [f64; 6] = [1., 1.25, 1.5, 1.5, 1.25, 1.];
        // Ignore isolated A and H file pawns.
        for (file, factor) in [B, C, D, E, F, G].into_iter().zip(factors) {
            let own_pawn_file = own_pawns & chess::get_file(file);
            let opponent_pawn_file = opponent_pawns & chess::get_file(file);

            let own_left = own_pawns & chess::get_file(file.left());
            let own_right = own_pawns & chess::get_file(file.right());
            let opponent_left = opponent_pawns & chess::get_file(file.left());
            let opponent_right = opponent_pawns & chess::get_file(file.right());

            let mut eval = 0;

            // Punish isolated pawns by half a pawn times a factor.
            if own_pawn_file.popcnt() != 0 && own_left.popcnt() == 0 && own_right.popcnt() == 0 {
                eval = -(((PAWN_VALUE / 2) as f64 * factor).round() as i64);
            }

            // Reward isolated opponent pawns by fourth a pawn times a factor.
            // Idea behind asymmetric evaluation is that it incentivises against having isolated
            // pawns but doesn't incentivice causing isolated pawns too much.
            if opponent_pawn_file.popcnt() != 0
                && opponent_left.popcnt() == 0
                && opponent_right.popcnt() == 0
            {
                eval += ((PAWN_VALUE / 4) as f64 * factor).round() as i64;
            }

            total_eval += eval;
        }

        // Store evaluation.
        self.isolated_pawns.insert(
            hash.0,
            PawnTableEntry::new(own_pawns, opponent_pawns, board.side_to_move(), total_eval),
        );

        total_eval
    }

    fn piece_count(board: Board, piece: Piece, color: Color) -> i64 {
        (board.pieces(piece) & board.color_combined(color)).popcnt() as i64
    }

    fn king_side(board: Board, file: File) -> bool {
        if board
            .king_square(board.side_to_move())
            .get_file()
            .to_index()
            <= 3
        {
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
