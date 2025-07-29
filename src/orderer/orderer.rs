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
    let captures = orderer_see::see_order_captures(board, &mut moves);

    moves.set_iterator_mask(!EMPTY);
    // Create with potentially too much capacity to avoid unnecessary allocations.
    let mut pv_moves = Vec::with_capacity(moves.len());
    let mut killer_moves = Vec::with_capacity(moves.len());
    let mut bad_captures = Vec::with_capacity(moves.len());
    let mut remaining_moves = Vec::with_capacity(moves.len());

    for mv in moves {
        if let Some(entry) = tt.get(board.make_move_new(mv).get_hash()) {
            match entry.flag {
                // PV move at higher or equal depth.
                TtEntryFlag::Exact if entry.depth >= depth => {
                    pv_moves.push(OrdererEntry::new(entry.value(board.side_to_move()), mv));
                }
                // Killer move at higher or equal depth.
                TtEntryFlag::Beta if entry.depth >= depth => {
                    killer_moves.push(OrdererEntry::new(entry.value(board.side_to_move()), mv));
                }
                _ => remaining_moves.push(mv),
            }
        } else {
            remaining_moves.push(mv);
        }
    }

    // Search principal variation move. Will be doubly in list, second search uses the hashed
    // result.
    if let Some(mv) = tt.get(board.get_hash()).map(|entry| entry.mv) {
        ret.push(mv);
    }

    // Sort PV hash moves.
    pv_moves.sort_unstable();
    // Search PV hash moves. Reverse since sort is ascending.
    ret.extend(pv_moves.iter().map(|entry| entry.mv).rev());

    // Search good and equal captures. Rev since it is sorted ascending.
    for entry in captures.iter().rev() {
        if entry.value >= 0 {
            ret.push(entry.mv);
        } else {
            // Store bad captures to add later.
            bad_captures.push(entry);
        }
    }

    // Sort killer moves.
    killer_moves.sort_unstable();
    // Search killer moves. Reverse since sort is ascending.
    ret.extend(killer_moves.iter().map(|entry| entry.mv).rev());

    // Search bad captures.
    ret.extend(bad_captures.iter().map(|entry| entry.mv));

    // Search unsorted remaining moves.
    ret.append(&mut remaining_moves);

    ret
}

/// SEE orders all available captures for quiescence search.
pub fn quiescence(board: &Board) -> impl Iterator<Item = ChessMove> {
    let mut moves = MoveGen::new_legal(board);
    moves.set_iterator_mask(orderer_masks::captures_mask(board));

    // Reverse since sort is ascending.
    orderer_see::see_order_captures(board, &mut moves)
        .into_iter()
        .map(|entry| entry.mv)
        .rev()
}
