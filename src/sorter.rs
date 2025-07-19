use chess::{BitBoard, Board, ChessMove, EMPTY, MoveGen, Square};

use crate::{
    evaluator::Evaluator,
    transposition_table::{TranspositionTable, TtEntryFlag},
};

pub struct Sorter {}

impl Sorter {
    pub fn all(
        board: &Board,
        depth: u16,
        tt: &TranspositionTable,
        board_ply: u16,
        search_ply: u16,
        pv_move: Option<ChessMove>,
    ) -> Vec<ChessMove> {
        let mut moves = MoveGen::new_legal(board);

        // +1 for principal variation move.
        let mut ret = Vec::with_capacity(moves.len() + 1);

        moves.set_iterator_mask(Sorter::captures_mask(board));
        let captures = Sorter::see_sort_captures(board, Vec::from_iter(&mut moves));

        moves.set_iterator_mask(!EMPTY);
        let mut pv_hash_moves = Vec::new();
        let mut killer_moves = Vec::new();
        let mut bad_captures = Vec::new();
        let mut remaining_moves = Vec::new();

        for mv in moves {
            let hash = board.make_move_new(mv).get_hash() + board_ply as u64 + search_ply as u64;
            if let Some(entry) = tt.get(hash) {
                match entry.flag {
                    // PV hash move at higher or equal depth.
                    TtEntryFlag::Exact if entry.depth >= depth => pv_hash_moves.push(mv),
                    // Killer move at higher or equal depth.
                    TtEntryFlag::Beta if entry.depth >= depth => killer_moves.push(mv),
                    _ => remaining_moves.push(mv),
                }
            } else {
                remaining_moves.push(mv);
            }
        }

        // Search principal variation move. Could be doubly in list, second search can use hashed
        // result.
        if let Some(mv) = pv_move {
            ret.push(mv);
        }

        // Search PV hash moves.
        ret.append(&mut pv_hash_moves);

        // Search good and equal captures.
        for (eval, capture) in captures {
            if eval >= 0 {
                ret.push(capture);
            } else {
                // Store bad captures to add later.
                bad_captures.push(capture);
            }
        }

        // Search killer moves.
        ret.append(&mut killer_moves);

        // Search bad captures.
        ret.extend(bad_captures);

        // Search remaining moves.
        ret.append(&mut remaining_moves);

        ret
    }

    pub fn quiescence(board: &Board) -> impl Iterator<Item = ChessMove> {
        let mut moves = MoveGen::new_legal(board);
        moves.set_iterator_mask(Sorter::captures_mask(board));

        Sorter::see_sort_captures(board, Vec::from_iter(&mut moves))
            .into_iter()
            .map(|(_, capture)| capture)
    }

    fn captures_mask(board: &Board) -> BitBoard {
        let captures = board.color_combined(!board.side_to_move());
        // En-passant moves are not included by the above mask since they don't land on the same
        // square of which they take.
        let en_passant = match board.en_passant() {
            Some(square) => BitBoard::from_square(square),
            None => EMPTY,
        };

        captures | en_passant
    }

    fn see_sort_captures(board: &Board, captures: Vec<ChessMove>) -> Vec<(i64, ChessMove)> {
        let mut sorted = Vec::with_capacity(captures.len());

        for capture in &captures {
            sorted.push((Sorter::see_capture(board, *capture), *capture));
        }
        sorted.sort_by(|(eval_a, _), (eval_b, _)| eval_a.cmp(eval_b).reverse());

        sorted
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
        // FIXME: rewrite this so it doesn't have to generate the moves (VERY inefficient).
        // It appears as if this is necessary since it's always a different position throughout
        // the recursion... :(
        let mut captures = MoveGen::new_legal(board);
        // FIXME: this excludes en-passant.
        captures.set_iterator_mask(Sorter::captures_mask(board) & BitBoard::from_square(square));

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
