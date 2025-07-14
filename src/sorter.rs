use chess::{BitBoard, Board, ChessMove, EMPTY, MoveGen, Square};
use fnv::FnvHashMap;

use crate::{
    evaluator::Evaluator,
    transposition_table::{TTEntry, TTEntryFlag},
};

pub struct Sorter {}

impl Sorter {
    pub fn all(board: &Board, tt: &FnvHashMap<u64, TTEntry>) -> Vec<ChessMove> {
        let mut moves = MoveGen::new_legal(&board);
        let mut ret = Vec::with_capacity(moves.len());

        moves.set_iterator_mask(Sorter::captures_mask(board));
        // FIXME: Vec needed because otherwise multiple mutable references.
        let captures = Vec::from_iter(Sorter::__quiescence(board, &mut moves));

        moves.set_iterator_mask(!EMPTY);
        let mut pv_hash_moves = Vec::new();
        let mut killer_moves = Vec::new();
        let mut remaining_moves = Vec::new();
        for mv in moves {
            if let Some(entry) = tt.get(&board.make_move_new(mv).get_hash()) {
                match entry.flag {
                    TTEntryFlag::Exact => pv_hash_moves.push(mv),
                    TTEntryFlag::Beta => killer_moves.push(mv),
                    _ => {}
                }
            } else {
                remaining_moves.push(mv);
            }
        }

        // Search hash PV moves.
        ret.append(&mut pv_hash_moves);

        // Search good and equal captures.
        let mut captures_idx = 0;
        for (idx, (eval, capture)) in captures.iter().enumerate() {
            if *eval < 0 {
                break;
            }

            // Store idx to skip to bad captures.
            captures_idx = idx;
            ret.push(*capture);
        }

        // Search killer moves.
        ret.append(&mut killer_moves);

        // Search bad captures.
        ret.extend(
            captures
                .iter()
                .skip(captures_idx)
                .map(|(_, capture)| capture),
        );

        // Search remaining moves.
        ret.append(&mut remaining_moves);

        ret
    }

    pub fn quiescence(board: &Board) -> impl Iterator<Item = ChessMove> {
        let mut captures = MoveGen::new_legal(&board);
        captures.set_iterator_mask(Sorter::captures_mask(board));

        // FIXME: Vec needed because otherwise dropped.
        let mut ret = Vec::with_capacity(captures.len());
        ret.extend(Sorter::__quiescence(board, &mut captures).map(|(_, capture)| capture));

        ret.into_iter()
    }

    fn __quiescence(
        board: &Board,
        captures: &mut MoveGen,
    ) -> impl Iterator<Item = (i64, ChessMove)> {
        let mut sorted = Vec::with_capacity(captures.len());
        sorted.extend(captures.map(|capture| {
            (
                Sorter::static_exchange_eval_capture(board, capture),
                capture,
            )
        }));

        // Sorts ascending, must thus be reversed.
        sorted.sort_by(|(eval_a, _), (eval_b, _)| eval_a.cmp(eval_b).reverse());

        sorted.into_iter()
    }

    fn captures_mask(board: &Board) -> BitBoard {
        let captures = board.color_combined(!board.side_to_move());
        // En passant moves are not included by the above mask since they don't land on the same
        // square of which they take.
        let en_passant = match board.en_passant() {
            Some(square) => BitBoard::from_square(square),
            None => EMPTY,
        };

        captures | en_passant
    }

    fn static_exchange_eval_capture(board: &Board, capture: ChessMove) -> i64 {
        let captured_value = Evaluator::piece_value(board.piece_on(capture.get_dest()).unwrap());

        captured_value
            - Sorter::static_exchange_eval(&board.make_move_new(capture), capture.get_dest())
    }

    fn static_exchange_eval(board: &Board, square: Square) -> i64 {
        let mut eval = 0;

        let smallest_attack = Sorter::smallest_attack(board, square);
        if let Some(smallest_attack) = smallest_attack {
            let captured_value =
                Evaluator::piece_value(board.piece_on(smallest_attack.get_dest()).unwrap());
            let new_eval = captured_value
                - Sorter::static_exchange_eval(&board.make_move_new(smallest_attack), square);

            if new_eval > 0 {
                eval = new_eval;
            }
        }

        eval
    }

    fn smallest_attack(board: &Board, square: Square) -> Option<ChessMove> {
        let mut captures = MoveGen::new_legal(&board);
        // FIXME: excludes en-passant but shouldn't matter too much.
        captures.set_iterator_mask(Sorter::captures_mask(board) & BitBoard::from_square(square));

        let mut smallest_attack = None;
        let mut smallest_attack_value = i64::MAX;
        for capture in captures {
            let new_value = Evaluator::piece_value(board.piece_on(capture.get_source()).unwrap());
            if new_value < smallest_attack_value {
                smallest_attack = Some(capture);
                smallest_attack_value = new_value;
            }
        }

        smallest_attack
    }
}
