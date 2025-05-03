//! This module contains the data model for GuiMessage, the commands sent from the GUI to the Engine.
//! The main enum is [`GuiMessage`] which encodes all the messages the GUI can send to the Shogi Engine.
//!
//! For full documenation about the protocol see
//! - [将棋所USIプロトコル](https://shogidokoro2.stars.ne.jp/usi.html)
//! - [The Universal Shogi Interface](http://hgm.nubati.net/usi.html)
use crate::format_vec;
use haitaka_types::Move;
use std::fmt;
use std::time::Duration;

pub const SFEN_STARTPOS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";

/// Messages sent from the GUI to the engine.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GuiMessage {
    /// `usi` - the first command sent to the engine to start the USI protocol.
    /// The engine should respond to this handshake with
    /// ```text
    /// id name <engine name>
    /// id author <engine developer>
    /// option <option parameters>
    /// (more options)
    /// usiok
    /// ```
    Usi,
    /// `debug` - turn debug mode on or off (not supported by all engines).
    /// ```text
    /// debug
    /// debug on
    /// debug off
    /// ```
    Debug(bool),
    /// `isready` - asks the engine whether it is ready to start a game or a search.
    /// This command should be sent after receiving `usiok` and after possibly resetting
    /// some options by sending `setoption` commands. The engine should respond with
    /// `readyok`. The GUI should not send other commands before receiving `readyok`.
    IsReady,
    /// `setoption` -sent to modify default engine option settings.
    /// ```text
    /// setoption name <option_name>
    /// setoption name <option_name> value <option_value>
    /// ```
    SetOption { name: String, value: Option<String> },
    /// `register` - registers the user to the engine. This is only required if the
    /// engine sent a `registration error` message at startup.
    /// ```text
    /// register later
    /// register name <user name> code <user registration code>
    /// ```
    Register {
        name: Option<String>,
        code: Option<String>,
    },
    /// `usinewgame` - indicates that the next search (to be started with `position` and `go`)
    /// will be from a new game. It should always be followed by an `isready` command.
    UsiNewGame,
    /// `position` - specifies the board position and optional sequence of moves.
    /// ```text
    /// position startpos
    /// position startpos moves <moves>
    /// position <SFEN>
    /// position <SFEN> moves <moves>
    /// ```
    Position {
        sfen: Option<String>,
        moves: Option<Vec<Move>>,
    },
    /// `go` - tells the engine to start its search for the best move, given the position.
    /// There are a number of optional subcommands to control the search and time settings
    /// of the engine. All of those should be sent in the same `go` command:
    /// ```text
    /// searchmoves <moves> - restrict the search to these (alternative) moves only
    /// ponder - start search in ponder mode (last move in the position command is ponder move)
    /// btime <ms> - black time left (millisecs)
    /// wtime <ms> - white time left
    /// binc <ms> - black time increment per move
    /// winc <ms> - white time increment per move
    /// byoyomi <ms> - byoyomi in millisecs (time per move after btime or wtime is 0)
    /// movestogo <n> - n moves until next time control (not used)
    /// depth <n> - search n plies deep only
    /// nodes <n> - search n nodes only
    /// movetime <ms> - search exactly so many millisecs
    /// infinite - search until receiving the `stop` command
    /// mate infinite - search for a forced mate until you find it
    /// mate <ms> - search for a forced mate but only up to so many millisecs
    /// ```
    Go(EngineParams),
    /// `stop` - tells the engine to stop as soon as possible.
    Stop,
    /// `ponderhit` - indicates that the engine opponent (the user) played the move predicted by the
    /// engine in the previous `bestmove` message. The engine should continue searching but
    /// switch from pondering to normal search.
    PonderHit,
    /// `gameover` - informs the engine that the game has ended with the specified result
    /// (specified from the engine's point of view).
    /// ```text
    /// gameover win
    /// gameover lose
    /// gameover draw
    /// ```
    GameOver(GameStatus),
    /// `quit` - tells the engine application to exit as soon as possible.
    Quit,
    /// This variant is a catch-all for messages that do not conform to the USI protocol.
    Unknown(String),
}

/// Represents status sent by "gameover" message.
///
/// Informs the engine that the game has ended with the specified result,
/// from the engine's own point or view.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum GameStatus {
    Win,
    Lose,
    Draw,
}

/// Engine search and time control parameters, sent by the "go" command.
///
/// Multiple parameters will and should be set in one "go" command.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct EngineParams {
    /// Restrict search to these moves only
    searchmoves: Option<Vec<Move>>,
    /// Start search in ponder mode
    ponder: bool,
    /// Black time left (ms)
    btime: Option<Duration>,
    /// White time left (ms)
    wtime: Option<Duration>,
    /// Black time increment per move (if greater than 0)
    binc: Option<Duration>,
    /// White time increment per move (if greater than 0)
    winc: Option<Duration>,
    /// Amount of time (ms) that each player is allowed to get negative on clock
    byoyomi: Option<Duration>,
    /// Number of moves (plies) until next time control. Only sent if greater than 0. (Not used in Shogi.)
    movestogo: Option<u16>,
    /// Search only this many plies.
    depth: Option<u16>,
    /// Search only this many nodes.
    nodes: Option<u32>,
    /// Search for mate in this amount of time (ms) or indefinitely long ("infinite")
    mate: Option<MateParam>,
    /// Search exactly this long (ms)
    movetime: Option<Duration>,
    /// Search until "stop" command is sent and received
    infinite: bool,
}

/// Mate paramater representing the "go mate x" command.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum MateParam {
    /// Find a mate in this many millisecs
    Timeout(Duration),
    /// Search indefinitely long until finding a forced mate
    Infinite,
}

impl EngineParams {
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn searchmoves(mut self, moves: Vec<Move>) -> Self {
        self.searchmoves = Some(moves);
        self
    }

    #[must_use]
    pub fn ponder(mut self) -> Self {
        self.ponder = true;
        self
    }

    #[must_use]
    pub fn btime(mut self, t: Duration) -> Self {
        self.btime = Some(t);
        self
    }

    #[must_use]
    pub fn wtime(mut self, t: Duration) -> Self {
        self.wtime = Some(t);
        self
    }

    #[must_use]
    pub fn binc(mut self, t: Duration) -> Self {
        self.binc = Some(t);
        self
    }

    #[must_use]
    pub fn winc(mut self, t: Duration) -> Self {
        self.winc = Some(t);
        self
    }

    #[must_use]
    pub fn byoyomi(mut self, t: Duration) -> Self {
        self.byoyomi = Some(t);
        self
    }

    #[must_use]
    pub fn movestogo(mut self, n: u16) -> Self {
        self.movestogo = Some(n);
        self
    }

    #[must_use]
    pub fn depth(mut self, n: u16) -> Self {
        self.depth = Some(n);
        self
    }

    #[must_use]
    pub fn nodes(mut self, n: u32) -> Self {
        self.nodes = Some(n);
        self
    }

    #[must_use]
    pub fn mate(mut self, t: MateParam) -> Self {
        self.mate = Some(t);
        self
    }

    #[must_use]
    pub fn movetime(mut self, t: Duration) -> Self {
        self.movetime = Some(t);
        self
    }

    #[must_use]
    pub fn infinite(mut self) -> Self {
        self.infinite = true;
        self
    }
}

// Note that the Display for GuiMessage does not add a terminating newline character.
// When actually sending protocol messages a writer should add the '\n'.

impl fmt::Display for GuiMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GuiMessage::Usi => write!(f, "usi"),
            GuiMessage::Debug(on) => match on {
                true => write!(f, "debug on"),
                false => write!(f, "debug off"),
            },
            GuiMessage::IsReady => write!(f, "isready"),
            GuiMessage::SetOption { name, value } => match value {
                Some(value) => write!(f, "setoption name {name} value {value}"),
                _ => write!(f, "setoption name {name}"),
            },
            GuiMessage::Register { name, code } => match (name, code) {
                (None, None) => write!(f, "register later"),
                (Some(name), None) => write!(f, "register name {name}"),
                (None, Some(code)) => write!(f, "register code {code}"),
                (Some(name), Some(code)) => write!(f, "register name {name} code {code}"),
            },
            GuiMessage::UsiNewGame => write!(f, "usinewgame"),
            GuiMessage::Position { sfen, moves } => match (sfen, moves) {
                (None, None) => write!(f, "position startpos"),
                (None, Some(moves)) => {
                    write!(f, "position startpos moves {}", format_vec!(moves))
                }
                (Some(sfen), None) => write!(f, "position sfen {}", sfen),
                (Some(sfen), Some(moves)) => {
                    write!(f, "position sfen {} moves {}", sfen, format_vec!(moves))
                }
            },
            GuiMessage::Go(params) => write!(f, "go{}", params), // params starts with space if non-empty
            GuiMessage::Stop => write!(f, "stop"),
            GuiMessage::PonderHit => write!(f, "ponderhit"),
            GuiMessage::GameOver(status) => write!(f, "gameover {}", status),
            GuiMessage::Quit => write!(f, "quit"),
            GuiMessage::Unknown(s) => write!(f, "UNKNOWN \"{}\"", s),
        }
    }
}

impl fmt::Display for GameStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            GameStatus::Win => write!(f, "win"),
            GameStatus::Lose => write!(f, "lose"),
            GameStatus::Draw => write!(f, "draw"),
        }
    }
}

impl fmt::Display for EngineParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut params = String::new();

        if self.ponder {
            params += " ponder";
        }
        if let Some(btime) = self.btime {
            params += &format!(" btime {}", btime.as_millis());
        }
        if let Some(wtime) = self.wtime {
            params += &format!(" wtime {}", wtime.as_millis());
        }
        if let Some(binc) = self.binc {
            params += &format!(" binc {}", binc.as_millis());
        }
        if let Some(winc) = self.winc {
            params += &format!(" winc {}", winc.as_millis());
        }
        if let Some(byoyomi) = self.byoyomi {
            params += &format!(" byoyomi {}", byoyomi.as_millis());
        }
        if let Some(movestogo) = self.movestogo {
            params += &format!(" movestogo {}", movestogo);
        }
        if let Some(depth) = self.depth {
            params += &format!(" depth {}", depth);
        }
        if let Some(nodes) = self.nodes {
            params += &format!(" nodes {}", nodes);
        }
        if let Some(ref mate) = self.mate {
            match mate {
                MateParam::Timeout(duration) => {
                    params += &format!(" mate {}", duration.as_millis());
                }
                MateParam::Infinite => {
                    params += " mate infinite";
                }
            }
        }
        if let Some(movetime) = self.movetime {
            params += &format!(" movetime {}", movetime.as_millis());
        }
        if self.infinite {
            params += " infinite";
        }
        if let Some(ref moves) = self.searchmoves {
            params += &format!(" searchmoves {}", format_vec!(moves));
        }

        // the output string will either be empty or start with a space
        write!(f, "{}", params)
    }
}
