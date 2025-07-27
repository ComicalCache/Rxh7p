use std::time::Duration;

use chess::ChessMove;

/// readyok.
pub fn ready_ok() {
    println!("readyok");
}

/// id.
pub fn id() {
    println!(
        "id name Rxh7+ V{}.{}.{}",
        env!("CARGO_PKG_VERSION_MAJOR"),
        env!("CARGO_PKG_VERSION_MINOR"),
        env!("CARGO_PKG_VERSION_PATCH")
    );
    println!("id author ComicalCache");
    println!("uciok");
}

/// bestmove.
pub fn best_move(mv: ChessMove, ponder_move: Option<ChessMove>) {
    let mut msg = format!("bestmove {mv}");

    if let Some(mv) = ponder_move {
        msg.push_str(format!(" ponder {mv}").as_str());
    }

    println!("{msg}");
}

/// General info message.
pub fn search_info(depth: u16, time: Duration, nodes: u64, pv: Vec<ChessMove>, score_cp: i64) {
    // FIXME: seldepth, refutation, currline and score mate should be sent.
    let mut msg = format!(
        "info depth {depth} time {} nodes {nodes} nps {} score cp {score_cp}",
        time.as_millis(),
        nodes / time.as_secs().max(1),
    );

    if !pv.is_empty() {
        msg.push_str(" pv");

        for mv in pv {
            msg.push_str(format!(" {mv}").as_str());
        }
    }

    println!("{msg}");
}
