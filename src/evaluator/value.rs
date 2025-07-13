use chess::Piece;

use crate::evaluator::{
    Evaluator, bishop::BISHOP_VALUE, king::KING_VALUE, knight::KNIGHT_VALUE, pawn::PAWN_VALUE,
    queen::QUEEN_VALUE, rook::ROOK_VALUE,
};

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

    pub(super) fn value(&self) -> i64 {
        let mut value = 0;

        self.__pawn_value(&mut value);
        self.__bishop_value(&mut value);
        self.__knight_value(&mut value);
        self.__rook_value(&mut value);
        self.__queen_value(&mut value);

        value
    }

    pub(super) fn mobility(&self) -> i64 {
        let mut total_mobility = 0;

        total_mobility += self.__pawn_mobility();
        total_mobility += self.__knight_mobility();
        total_mobility += self.__bishop_mobility();

        let rook_mobility = self.__rook_mobility();
        let queen_mobility = self.__queen_mobility();
        // Don't incentivise early rook and queen development.
        // FIXME: Using only remaining pieces is not a great metric.
        if self.board.color_combined(self.color).popcnt() > 14 {
            total_mobility += rook_mobility / 8;
            total_mobility += queen_mobility / 8;
        } else if self.board.color_combined(self.color).popcnt() > 10 {
            total_mobility += rook_mobility / 4;
            total_mobility += queen_mobility / 4;
        } else {
            total_mobility += rook_mobility;
            total_mobility += queen_mobility;
        }

        total_mobility += self.__king_mobility();

        total_mobility
    }
}
