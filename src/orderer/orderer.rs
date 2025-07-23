use std::collections::BinaryHeap;

use chess::{Board, ChessMove, EMPTY, MoveGen};

use crate::{
    orderer::{orderer_entry::OrdererEntry, orderer_masks, orderer_see},
    tt::{TT, TtEntryFlag},
};

/// Orders all available moves by:<br>
/// 1. TT move.
/// 2. PV moves.
/// 3. Good and equal captures.
/// 4. Killer moves.
/// 5. Bad captures.
/// 6. Remaining moves (random order).
pub fn all(board: &Board, depth: u16, tt: &TT) -> Vec<ChessMove> {
    let mut moves = MoveGen::new_legal(board);

    // Plus one for principal variation move.
    let mut ret = Vec::with_capacity(moves.len() + 1);

    // Get all captures and order them.
    moves.set_iterator_mask(orderer_masks::captures_mask(board));
    let captures = orderer_see::see_order_captures(board, Vec::from_iter(&mut moves));

    moves.set_iterator_mask(!EMPTY);
    let mut pv_moves = BinaryHeap::new();
    let mut killer_moves = BinaryHeap::new();
    let mut bad_captures = BinaryHeap::new();
    let mut remaining_moves = Vec::new();

    for mv in moves {
        if let Some(entry) = tt.get(board.make_move_new(mv).get_hash()) {
            match entry.flag {
                // PV move at higher or equal depth.
                TtEntryFlag::Exact if entry.depth >= depth => {
                    pv_moves.push(OrdererEntry::new(entry.value(board.side_to_move()), mv))
                }
                // Killer move at higher or equal depth.
                TtEntryFlag::Beta if entry.depth >= depth => {
                    killer_moves.push(OrdererEntry::new(entry.value(board.side_to_move()), mv))
                }
                _ => remaining_moves.push(mv),
            }
        } else {
            remaining_moves.push(mv);
        }
    }

    // Search principal variation move. Will be doubly in list, second search uses the hashed
    // result.
    if let Some(entry) = tt.get(board.get_hash()) {
        ret.push(entry.mv);
    }

    // Search PV hash moves.
    ret.extend(pv_moves.iter().map(|entry| entry.mv));

    // Search good and equal captures.
    for entry in captures {
        if entry.value >= 0 {
            ret.push(entry.mv);
        } else {
            // Store bad captures to add later.
            bad_captures.push(entry);
        }
    }

    // Search killer moves.
    ret.extend(killer_moves.iter().map(|entry| entry.mv));

    // Search bad captures.
    ret.extend(bad_captures.iter().map(|entry| entry.mv));

    // Search remaining moves.
    ret.append(&mut remaining_moves);

    ret
}

/// SEE orders all available captures for quiescence search.
pub fn quiescence(board: &Board) -> impl Iterator<Item = ChessMove> {
    let mut moves = MoveGen::new_legal(board);
    moves.set_iterator_mask(orderer_masks::captures_mask(board));

    orderer_see::see_order_captures(board, Vec::from_iter(&mut moves))
        .into_iter()
        .map(|entry| entry.mv)
}
