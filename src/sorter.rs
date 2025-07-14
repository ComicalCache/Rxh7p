use std::i64;

use chess::{BitBoard, Board, ChessMove, EMPTY, MoveGen, Square};

use crate::evaluator::Evaluator;

pub struct Sorter {}

impl Sorter {
    pub fn all(board: &Board) -> impl Iterator<Item = ChessMove> {
        MoveGen::new_legal(board)
    }

    pub fn quiescence(board: &Board) -> impl Iterator<Item = ChessMove> {
        let captures = Sorter::captures(board);
        let mut sorted = Vec::with_capacity(captures.len());

        sorted.extend(captures.map(|capture| (Sorter::see_capture(board, capture), capture)));
        sorted.sort_by(|(eval_a, _), (eval_b, _)| eval_a.cmp(eval_b).reverse());

        sorted.into_iter().map(|(_, capture)| capture)
    }

    pub fn captures(board: &Board) -> MoveGen {
        let mut moves = MoveGen::new_legal(board);

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

    fn see_capture(board: &Board, capture: ChessMove) -> i64 {
        let captured_value = Evaluator::piece_value(board.piece_on(capture.get_dest()).unwrap());

        captured_value - Sorter::see(&board.make_move_new(capture), capture.get_dest())
    }

    fn see(board: &Board, square: Square) -> i64 {
        let mut eval = 0;

        let smallest_attack = Sorter::smallest_attack(board, square);
        if let Some(smallest_attack) = smallest_attack {
            let captured_value =
                Evaluator::piece_value(board.piece_on(smallest_attack.get_dest()).unwrap());
            let new_eval =
                captured_value - Sorter::see(&board.make_move_new(smallest_attack), square);
            if new_eval > 0 {
                eval = new_eval;
            }
        }

        eval
    }

    fn smallest_attack(board: &Board, square: Square) -> Option<ChessMove> {
        // FIXME: rewrite this so it doesn't have to generate the moves (VERY inefficient)
        let mut captures = Sorter::captures(board);

        // excludes en-passant but doesn't matter for now
        captures.set_iterator_mask(BitBoard::from_square(square));

        let mut smallest_attack = None;
        let mut smallest_attack_value = i64::MAX;
        for capture in captures {
            let curr_value = Evaluator::piece_value(board.piece_on(capture.get_source()).unwrap());
            if curr_value < smallest_attack_value {
                smallest_attack = Some(capture);
                smallest_attack_value = curr_value;
            }
        }

        smallest_attack
    }
}
