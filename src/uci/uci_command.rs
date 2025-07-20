use std::{str::FromStr, time::Duration};

use chess::{Board, ChessMove};

/// A struct containing the received configuration by a go command.
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

/// Type of UCI command.
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
    /// Parses the parameters of a UCI position command.
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

    /// Parses the parameters of a UCI go command.
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
    /// Creates a UciCommand (including parsed parameters) from a string.
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
