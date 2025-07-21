use std::collections::BinaryHeap;

use chess::{BitBoard, Board, ChessMove, MoveGen, Square};

use crate::{
    evaluator::Evaluator,
    orderer::{Orderer, orderer_entry::OrdererEntry},
};

impl Orderer {
    /// Orders all captures according to their SEE.
    pub(super) fn see_order_captures(
        board: &Board,
        captures: Vec<ChessMove>,
    ) -> BinaryHeap<OrdererEntry> {
        let mut ordered = BinaryHeap::with_capacity(captures.len());

        for capture in &captures {
            ordered.push(OrdererEntry::new(
                Orderer::see_capture(board, *capture),
                *capture,
            ));
        }

        ordered
    }

    /// Performs SEE on a capture.
    fn see_capture(board: &Board, capture: ChessMove) -> i64 {
        let captured_value = Evaluator::piece_value(board.piece_on(capture.get_dest()).unwrap());

        captured_value - Orderer::see(&board.make_move_new(capture), capture.get_dest())
    }

    /// Performs SEE on a square.
    fn see(board: &Board, square: Square) -> i64 {
        let mut eval = 0;

        let smallest_attack = Orderer::smallest_attack(board, square);
        if let Some(smallest_attack) = smallest_attack {
            let captured_value =
                Evaluator::piece_value(board.piece_on(smallest_attack.get_dest()).unwrap());
            let new_eval =
                captured_value - Orderer::see(&board.make_move_new(smallest_attack), square);
            if new_eval > 0 {
                eval = new_eval;
            }
        }

        eval
    }

    /// Finds the smalles attacking piece.
    fn smallest_attack(board: &Board, square: Square) -> Option<ChessMove> {
        // FIXME: rewrite this so it doesn't have to generate the moves (VERY inefficient).
        // It appears as if this is necessary since it's always a different position throughout
        // the recursion... :(
        let mut captures = MoveGen::new_legal(board);
        // FIXME: this excludes en-passant.
        captures.set_iterator_mask(Orderer::captures_mask(board) & BitBoard::from_square(square));

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
