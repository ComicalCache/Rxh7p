use std::{str::FromStr, sync::mpsc::Sender, time::Duration};

use chess::{Board, ChessMove};

pub struct GoCommandConfig {
    /// Only search moves in this list.
    pub searchmoves: Vec<ChessMove>,
    /// Ponder on the last move received by the position command.
    pub ponder: bool,
    /// Time left for the white player.
    pub wtime: Option<Duration>,
    /// Time left for the black player.
    pub btime: Option<Duration>,
    /// Time increment for the white player.
    pub winc: Option<Duration>,
    /// Time increment for the black player.
    pub binc: Option<Duration>,
    /* TODO: implement moves_to_go
    /// How many moves until time increment.
    pub moves_to_go: u64,
    */
    /// Search x plies.
    pub depth: Option<u16>,
    /// Search x nodes.
    pub nodes: Option<u64>,
    /* TODO: implement search for mate.
    /// Search for mate in x moves.
    pub mate: u64,
    */
    /// Search for exactly the duration.
    pub move_time: Option<Duration>,
}

pub enum UciCommand {
    Invalid,
    Uci,
    // TODO: implement debug command.
    IsReady,
    // TODO: implement setoption command.
    // TODO: implement register command.
    UciNewGame,
    Position(Board, Option<Vec<ChessMove>>),
    Go(GoCommandConfig),
    Stop,
    Ponderhit,
    Quit,
}

impl UciCommand {
    fn parse_position<'a>(mut args: impl Iterator<Item = &'a str>) -> UciCommand {
        // Parse start board for position.
        let board = match args.next() {
            Some("fen") => {
                let mut fen_str = args
                    .next()
                    .expect("Expected a valid fen string from the position fen command")
                    .to_string();

                for _ in 0..5 {
                    fen_str.push(' ');
                    fen_str.push_str(
                        args.next()
                            .expect("Expected a valid fen string from the position fen command"),
                    );
                }

                Board::from_str(&fen_str)
                    .expect("Expected a valid fen string from the position fen command")
            }
            Some("startpos") => Board::default(),
            _ => return UciCommand::Invalid,
        };

        // Invalid position command.
        if let Some(moves) = args.next() {
            if moves != "moves" {
                return UciCommand::Invalid;
            }
        } else {
            return UciCommand::Position(board, None);
        }

        let mut moves = Vec::new();
        for mv in args {
            moves.push(
                ChessMove::from_str(mv).expect("Expected a valid move from the position command"),
            );
        }

        UciCommand::Position(board, Some(moves))
    }

    fn parse_go<'a>(mut args: impl Iterator<Item = &'a str>) -> UciCommand {
        // By default search infinitely.
        let mut config = GoCommandConfig {
            searchmoves: Vec::new(),
            ponder: false,
            wtime: None,
            btime: None,
            winc: None,
            binc: None,
            depth: None,
            nodes: None,
            move_time: None,
        };

        let num = |arg: Option<&str>, cmd_name: &str, kind: &str| {
            arg.unwrap_or_else(|| panic!("Expected seconds from go {cmd_name} command"))
                .parse()
                .unwrap_or_else(|_| {
                    panic!("Expected a valid number of {kind} from the go {cmd_name} command")
                })
        };

        while let Some(arg) = args.next() {
            match arg {
                "searchmoves" => {
                    for mv in args.by_ref() {
                        config.searchmoves.push(
                            ChessMove::from_str(mv)
                                .expect("Expected a valid move from the go searchmoves command"),
                        );
                    }
                }
                "ponder" => config.ponder = true,
                "wtime" => {
                    config.wtime = Some(Duration::from_millis(num(args.next(), "wtime", "seconds")))
                }
                "btime" => {
                    config.btime = Some(Duration::from_millis(num(args.next(), "btime", "seconds")))
                }
                "winc" => {
                    config.winc = Some(Duration::from_millis(num(args.next(), "winc", "seconds")))
                }
                "binc" => {
                    config.binc = Some(Duration::from_millis(num(args.next(), "binc", "seconds")))
                }
                // TODO: implement moves to go.
                "movestogo" => {}
                "depth" => config.depth = Some(num(args.next(), "depth", "plies") as u16),
                "nodes" => config.nodes = Some(num(args.next(), "nodes", "nodes")),
                "movetime" => {
                    config.move_time = Some(Duration::from_millis(num(
                        args.next(),
                        "movetime",
                        "milliseconds",
                    )))
                }
                // Just ignore the infinite token since the default is infinite.
                "infinite" => {}
                _ => return UciCommand::Invalid,
            }
        }

        UciCommand::Go(config)
    }
}

impl From<&str> for UciCommand {
    fn from(value: &str) -> Self {
        use UciCommand::*;

        let mut split = value.split_whitespace();
        if let Some(command) = split.next() {
            match command {
                "uci" => Uci,
                "isready" => IsReady,
                "ucinewgame" => UciNewGame,
                "position" => UciCommand::parse_position(split),
                "go" => UciCommand::parse_go(split),
                "stop" => Stop,
                "ponderhit" => Ponderhit,
                "quit" => Quit,
                _ => Invalid,
            }
        } else {
            Invalid
        }
    }
}

pub struct Uci {
    uci_tx: Sender<UciCommand>,
    stop_tx: Sender<()>,
    ponderhit_tx: Sender<()>,
}

impl Uci {
    pub fn new(uci_tx: Sender<UciCommand>, stop_tx: Sender<()>, ponderhit_tx: Sender<()>) -> Self {
        Uci {
            uci_tx,
            stop_tx,
            ponderhit_tx,
        }
    }

    pub fn start(&mut self) {
        let stdin = std::io::stdin();

        loop {
            let mut input = String::new();
            stdin
                .read_line(&mut input)
                .expect("Failed to read from stdin");

            let command = UciCommand::from(input.trim());

            let send = |cmd| {
                self.uci_tx
                    .send(cmd)
                    .expect("Failed to send command to main thread")
            };

            match command {
                // Don't propagate invalid commands to the engine.
                UciCommand::Invalid => continue,
                UciCommand::Uci => Uci::id(),
                // Send here since main is busy.
                UciCommand::Stop => self
                    .stop_tx
                    .send(())
                    .expect("Failed to send stop message to engine"),
                UciCommand::Ponderhit => self
                    .ponderhit_tx
                    .send(())
                    .expect("Failed to send ponderhit message to engine"),
                // No need to send quit command, loop in main quits when this loop ends since the
                // tx value gets dropped.
                UciCommand::Quit => break,
                _ => send(command),
            }
        }
    }

    pub fn ok() {
        println!("readyok");
    }

    fn id() {
        println!("id name Rxh7+");
        println!("id author ComicalCache");
        println!("uciok");
    }

    pub fn best_move(mv: ChessMove, ponder_moves: Option<Vec<ChessMove>>) {
        let mut msg = format!("bestmove {mv}");

        if let Some(ponder_moves) = ponder_moves {
            msg.push_str(" ponder");
            for ponder_mv in ponder_moves {
                msg.push_str(format!(" {ponder_mv}").as_str());
            }
        }

        println!("{msg}");
    }

    pub fn search_info(depth: u16, time: Duration, nodes: u64, pv: Vec<ChessMove>, score_cp: i64) {
        // FIXME: seldepth, nps, refutation, currline and score mate should be sent.
        let mut msg = format!(
            "info depth {depth} time {} nodes {nodes} score cp {score_cp}",
            time.as_millis()
        );

        if !pv.is_empty() {
            msg.push_str(" pv");

            for mv in pv {
                msg.push_str(format!(" {mv}").as_str());
            }
        }

        println!("{msg}");
    }

    pub fn curr_move_info(curr_move: ChessMove, curr_move_number: u64) {
        println!("info currmove {curr_move} currmovenumber {curr_move_number}");
    }
}
