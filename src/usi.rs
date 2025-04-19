// usi.rs

use chrono::Duration;
use haitaka_types::Move;
use pest::error::Error as PestError;
use std::fmt::{Display, Formatter, Result};

use crate::parser::Rule;

/// A vector of UsiMessage instances.
pub type UsiMessageList = Vec<UsiMessage>;

/// An enumeration type with representations for all USI protocol messages
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UsiMessage {
    /// Messages sent from the GUI to the engine.
    UsiGuiToEngine(GuiMessage),

    /// Messages sent from the engine to the GUI.
    UsiEngineToGui(EngineMessage),

    /// The Unknown Message (probably Lost in Translation)
    Unknown(String, Option<PestError<Rule>>),
}

/// Messages sent from the GUI to the engine.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GuiMessage {
    Usi,
    Debug(bool),
    IsReady,
    Register {
        later: bool,
        name: Option<String>,
        code: Option<String>,
    },
    Position {
        startpos: bool,
        sfen: Option<String>,
        moves: Option<Vec<Move>>,
    },
    SetOption {
        name: String,
        value: Option<String>,
    },
    UsiNewGame,
    Stop,
    PonderHit,
    Quit,
    Go {
        time_control: Option<UsiTimeControl>,
        search_control: Option<UsiSearchControl>,
    },
}

/*

    "option name Nullmove type check default true\n"
    "option name Selectivity type spin default 2 min 0 max 4\n"
    "option name Style type combo default Normal var Solid var Normal var Risky\n"
    "option name LearningFile type filename default /shogi/my-shogi-engine/learn.bin"
    "option name ResetLearning type button\n"

*/

/// Messages sent from the engine to the GUI.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum EngineMessage {
    Id {
        name: Option<String>,
        author: Option<String>,
    },
    UsiOk,
    ReadyOk,
    BestMove {
        best_move: Move,
        ponder: Option<Move>,
    },
    CopyProtection(StatusCheck),
    Registration(StatusCheck),
    Option(UsiOptionType),
    Info(Vec<UsiInfo>),
}

/// Represents the copy protection or registration state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum StatusCheck {
    /// Signifies the engine is checking the copy protection or registration.
    Checking,

    /// Signifies the copy protection or registration has been validated.
    Ok,

    /// Signifies error in copy protection or registratin validation.
    Error,
}

impl Display for StatusCheck {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            StatusCheck::Checking => write!(f, "checking"),
            StatusCheck::Ok => write!(f, "ok"),
            StatusCheck::Error => write!(f, "error"),
        }
    }
}

/// Time control settings (send by the `go` message).
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UsiTimeControl {
    /// The `go ponder` message.
    Ponder,

    /// The `go infinite` message. Search until the `stop` command is received. Do not exit search earlier.
    Infinite,

    /// The information about the game's time controls.
    TimeLeft {
        /// White's time on the clock, in milliseconds.
        white_time: Option<Duration>,

        /// Black's time on the clock, in milliseconds.
        black_time: Option<Duration>,

        /// White's increment per move, in milliseconds.
        white_increment: Option<Duration>,

        /// Black's increment per move, in milliseconds.
        black_increment: Option<Duration>,

        /// The number of moves to go to the next time control. Only set if greater than 0.
        /// If not set, and wtime and btime are set, then it's sudden death. This option is
        /// never used in Shogi. Instead, byoyomi is used.
        moves_to_go: Option<u8>,

        /// (Shogidokoro) In milliseconds. Max allowed negative time after time's up.
        /// Resets to 0 with every move. If surpassed, the game is lost.
        byoyomi: Option<Duration>,
    },

    /// Specifies how much time exactly the engine should think about the move, in milliseconds.
    MoveTime(Duration),
}

impl UsiTimeControl {
    /// Return a UsiTimeControl::TimeLeft instance with all fields set to None.
    pub fn time_left() -> UsiTimeControl {
        UsiTimeControl::TimeLeft {
            white_time: None,
            black_time: None,
            white_increment: None,
            black_increment: None,
            moves_to_go: None,
            byoyomi: None,
        }
    }
}

/// Search control settings (set by `go` message).
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct UsiSearchControl {
    /// Limit the search to these moves.
    pub searchmoves: Vec<Move>,

    /// Search for mate in this many moves (plies).
    pub mate: Option<u16>,

    /// Search this many plies.
    pub depth: Option<u16>,

    /// Search this many nodes (positions).
    pub nodes: Option<u64>,
}

impl Default for UsiSearchControl {
    fn default() -> Self {
        UsiSearchControl {
            searchmoves: Vec::new(),
            mate: None,
            depth: None,
            nodes: None,
        }
    }
}

impl UsiSearchControl {
    pub fn is_active(&self) -> bool {
        // cannot be `pub const fn` because Vec `is_empty` is not yet stable
        !self.searchmoves.is_empty()
            || self.mate.is_some()
            || self.depth.is_some()
            || self.nodes.is_some()
    }
}

/// USI option type.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UsiOptionType {
    /// The option of type `check` (a boolean).
    Check {
        /// The name of the option.
        name: String,

        /// The default value of this `bool` property.
        default: Option<bool>,
    },

    /// The option of type `spin` (a signed integer).
    Spin {
        /// The name of the option.
        name: String,

        /// The default value of this integer property.
        default: Option<i64>,

        /// The minimal value of this integer property.
        min: Option<i64>,

        /// The maximal value of this integer property.
        max: Option<i64>,
    },

    /// The option of type `combo` (a list of strings).
    Combo {
        /// The name of the option.
        name: String,

        /// The default value for this list of strings.
        default: Option<String>,

        /// The list of acceptable strings.
        var: Vec<String>,
    },

    /// The option of type `button` (an action).
    Button {
        /// The name of the option.
        name: String,
    },

    /// The option of type `string`.
    String {
        /// The name of the option.
        name: String,

        /// The default value of this string option.
        default: Option<String>,
    },

    /// The option of type `filename`. Similar to `string` but may be presented as file browser in a GUI.
    Filename {
        name: String,
        default: Option<String>,
    },
}

// TODO: Review this.
// Isn't it a bit weird to have the UsiInfo as a Vec<UsiInfo>
// rather than as a struct with fields?

/// Various info messages.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum UsiInfo {
    /// The `info depth` message.
    Depth(u16),

    /// The `info seldepth` message.
    SelDepth(u16),

    /// The `info time` message.
    Time(Duration),

    /// The `info nodes` message.
    Nodes(u64),

    /// The `info pv` message (best line move sequence).
    Pv(Vec<Move>),

    /// The `info pv ... multipv` message (the pv line number in a multi pv sequence).
    MultiPv(u16),

    /// The `info score ...` message.
    Score {
        /// The score in centipawns.
        cp: Option<i32>,

        /// Mate coming up in this many moves. Negative value means the engine is getting mated.
        mate: Option<i16>,

        /// The value sent is the lower bound.
        lowerbound: Option<bool>,

        /// The value sent is the upper bound.
        upperbound: Option<bool>,
    },

    /// The `info currmove` message (current move).
    CurrMove(Move),

    /// The `info currmovenum` message (current move number).
    CurrMoveNum(u16),

    /// The `info hashfull` message (occupancy of hash tables in permills, from 0 to 1000).
    HashFull(u16),

    /// The `info nps` message (nodes per second).
    Nps(u64),

    /// The `info cpuload` message (CPU load in permills).
    CpuLoad(u16),

    /// The `info string` message (a string the GUI should display).
    String(String),

    /// The `info refutation` message (the first move is the move being refuted).
    Refutation(Vec<Move>),

    /// The `info currline` message (current line being calculated on a CPU).
    CurrLine {
        /// The CPU number calculating this line.
        cpu_nr: Option<u16>,

        /// The line being calculated.
        line: Vec<Move>,
    },

    /// Any other info line in the format `(name, value)`.
    Any(String, String),
}

// Display trait implementations

impl Display for UsiMessage {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(
            f,
            "{}",
            match self {
                UsiMessage::UsiGuiToEngine(msg) => format!("{}", msg),
                UsiMessage::UsiEngineToGui(msg) => format!("{}", msg),
                UsiMessage::Unknown(msg, opt_err) => {
                    if let Some(err) = opt_err {
                        format!("ERROR msg='{}' error='{}'", msg, err)
                    } else {
                        format!("unknown msg '{}'", msg)
                    }
                }
            }
        )
    }
}

impl Display for GuiMessage {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            GuiMessage::Usi => write!(f, "usi"),
            GuiMessage::Debug(on) => write!(f, "debug {}", if *on { "on" } else { "off" }),
            GuiMessage::IsReady => write!(f, "isready"),
            GuiMessage::Register { later, name, code } => {
                if *later {
                    write!(f, "register later")
                } else {
                    let mut s = String::from("register ");
                    if let Some(n) = name {
                        s += &format!("name {}", n);
                    }
                    if let Some(c) = code {
                        if name.is_some() {
                            s += &format!(" code {}", c);
                        } else {
                            s += &format!("code {}", c);
                        }
                    }
                    write!(f, "{}", s)
                }
            }
            GuiMessage::Position {
                startpos,
                sfen,
                moves,
            } => {
                let mut s = String::from("position ");
                if *startpos {
                    s += &format!("startpos");
                } else if let Some(sfen) = sfen {
                    s += &format!("sfen {}", sfen);
                } else {
                    assert!(false, "GuiMessage::Position misses both startpos and sfen");
                }
                if let Some(moves) = moves {
                    s += &format!(
                        "moves {}",
                        moves
                            .iter()
                            .map(|m| m.to_string())
                            .collect::<Vec<_>>()
                            .join(" ")
                    );
                }
                write!(f, "{}", s)
            }
            GuiMessage::SetOption { name, value } => {
                if let Some(v) = value {
                    write!(f, "setoption name {} value {}", name, v)
                } else {
                    write!(f, "setoption name {}", name)
                }
            }
            GuiMessage::UsiNewGame => write!(f, "usinewgame"),
            GuiMessage::Stop => write!(f, "stop"),
            GuiMessage::PonderHit => write!(f, "ponderhit"),
            GuiMessage::Quit => write!(f, "quit"),
            GuiMessage::Go {
                time_control,
                search_control,
            } => {
                let mut s = String::from("go");
                if let Some(tc) = time_control {
                    s += &format!(" {}", tc);
                }
                if let Some(sc) = search_control {
                    s += &format!(" {}", sc);
                }
                write!(f, "{}", s)
            }
        }
    }
}

impl Display for UsiTimeControl {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            _ => write!(f, ""),
        }
    }
}

impl Display for UsiSearchControl {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            _ => write!(f, ""),
        }
    }
}

impl Display for EngineMessage {
    fn fmt(&self, f: &mut Formatter) -> Result {
        match self {
            EngineMessage::Id { name, author } => {
                if let Some(n) = name {
                    write!(f, "id name {}", n)
                } else if let Some(a) = author {
                    write!(f, "id author {}", a)
                } else {
                    // reachable???
                    write!(f, "id")
                }
            }
            EngineMessage::UsiOk => write!(f, "usiok"),
            EngineMessage::ReadyOk => write!(f, "readyok"),
            EngineMessage::BestMove { best_move, ponder } => {
                let mut s = format!("bestmove {}", best_move);
                if let Some(p) = ponder {
                    s += &format!(" ponder {}", p);
                }
                write!(f, "{}", s)
            }
            EngineMessage::CopyProtection(state) => write!(f, "copyprotection {}", *state),
            EngineMessage::Registration(state) => write!(f, "register {}", *state),
            EngineMessage::Option(option) => write!(f, "option {}", *option),
            EngineMessage::Info(info) => {
                let info_str = info
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join(" ");
                write!(f, "info {}", info_str)
            }
        }
    }
}

impl Display for UsiOptionType {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "option")
    }
}

impl Display for UsiInfo {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "info")
    }
}

/*
impl Display for SFENPosition {
    fn fmt(&self, f: &mut Formatter) -> Result {
        if self.moves.len() > 0 {
            let moves_str = self.moves.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(" ");
            write!(f, "{} moves {}", self.sfen, moves_str)
        } else {
            write!(f, "{}", self.sfen)
        }
    }
}
*/
