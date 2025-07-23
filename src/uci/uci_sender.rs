use std::{sync::mpsc::Receiver, time::Duration};

use chess::ChessMove;

/// A UCI message sent by the engine.
pub enum UciSenderMessage {
    /// bestmove message.
    BestMove(ChessMove, Option<Vec<ChessMove>>),
    /// General info message.
    SearchInfo(u16, Duration, u64, Vec<ChessMove>, i64),
}

/// Struct acting as a UCI sender. It runs in its own thread and communicates with the engine via
/// message passing.
pub struct UciSender {
    /// Channel for receiving messages from the engine.
    message_rx: Receiver<UciSenderMessage>,
}

impl UciSender {
    /// Creates a new UCI sender.
    pub fn new(message_rx: Receiver<UciSenderMessage>) -> Self {
        UciSender { message_rx }
    }

    /// Main loop, receiving messages from the engine and propagating them to UCI.
    pub fn start(&mut self) {
        while let Ok(msg) = self.message_rx.recv() {
            match msg {
                UciSenderMessage::BestMove(mv, ponder_moves) => {
                    UciSender::best_move(mv, ponder_moves)
                }
                UciSenderMessage::SearchInfo(depth, time, nodes, pv, score_cp) => {
                    UciSender::search_info(depth, time, nodes, pv, score_cp)
                }
            }
        }
    }

    /// readyok.
    pub fn ready_ok() {
        println!("readyok");
    }

    /// id.
    pub fn id() {
        println!("id name Rxh7+ V1.2");
        println!("id author ComicalCache");
        println!("uciok");
    }

    /// bestmove.
    fn best_move(mv: ChessMove, ponder_moves: Option<Vec<ChessMove>>) {
        let mut msg = format!("bestmove {mv}");

        if let Some(ponder_moves) = ponder_moves {
            msg.push_str(" ponder");
            for ponder_mv in ponder_moves {
                msg.push_str(format!(" {ponder_mv}").as_str());
            }
        }

        println!("{msg}");
    }

    /// General info message.
    fn search_info(depth: u16, time: Duration, nodes: u64, pv: Vec<ChessMove>, score_cp: i64) {
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
}
