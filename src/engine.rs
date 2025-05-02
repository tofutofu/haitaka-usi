//! This module contains the data-model for EngineMessage, the commands sent from Engine to GUI.
use crate::format_vec;
use haitaka_types::Move;
use std::fmt;
use std::time::Duration;

/// Messages sent from the Shogi Engine to the GUI.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum EngineMessage {
    Id(IdParams),
    UsiOk,
    ReadyOk,
    BestMove(BestMoveParams),
    CheckMate(CheckMateParams),
    CopyProtection(StatusCheck),
    Registration(StatusCheck),
    Option(OptionParam),
    Info(Vec<InfoParam>),
    Unknown(String),
}

/// Represents content of "id" message ("id name..." or "id author ...").
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum IdParams {
    Name(String),
    Author(String),
}

/// Represents payload of "bestmove" message.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum BestMoveParams {
    BestMove {
        bestmove: Move,
        ponder: Option<Move>,
    },
    Win,
    Resign,
}

/// Represents payload of "checkmate" message, sent after a "go mate" search terminates.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum CheckMateParams {
    /// Main line of checkmate solution
    Mate(Vec<Move>),
    /// No forced mate exists
    NoMate,
    /// Search for a forced mate timed out and was inconclusive
    TimeOut,
    /// Search for forced mates is not implemented
    NotImplemented,
}

/// Represents copy protection or registration state.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub enum StatusCheck {
    /// Signifies the engine is checking the copy protection or registration.
    Checking,

    /// Signifies copy protection or registration has been validated.
    Ok,

    /// Signifies error in copy protection or registratin validation.
    Error,
}

/// Represents contents of the "option" message.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum OptionParam {
    Check {
        name: String,
        default: Option<bool>,
    },
    Spin {
        name: String,
        default: Option<i32>,
        min: Option<i32>,
        max: Option<i32>,
    },
    Combo {
        name: String,
        default: Option<String>,
        vars: Vec<String>,
    },
    Button {
        name: String,
        // default: Option<String>
    },
    String {
        name: String,
        default: Option<String>,
    },
    Filename {
        name: String,
        default: Option<String>,
    },
}

/// Represents possible payloads of the "info" message.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum InfoParam {
    /// The `info depth` message.
    Depth(u16),

    /// The `info seldepth` message.
    SelDepth(u16),

    /// The `info time` message.
    Time(Duration),

    /// The `info nodes` message.
    Nodes(u64),

    /// The `info pv` message (principal variation, best line).
    Pv(Vec<Move>),

    /// The `info pv ... multipv` message (the pv line number in a multi pv sequence).
    MultiPv(u16),

    /// The 'info score cp ...' message.
    ScoreCp(i32, ScoreBound),

    /// The info score mate ...' message.
    ScoreMate(Option<i32>, ScoreBound),

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
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum ScoreBound {
    MatePlus,
    MateMin,
    Exact,
    Lower,
    Upper,
}

impl fmt::Display for EngineMessage {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            EngineMessage::Id(params) => write!(f, "id {}", params),
            EngineMessage::UsiOk => write!(f, "usiok"),
            EngineMessage::ReadyOk => write!(f, "readyok"),
            EngineMessage::BestMove(params) => write!(f, "bestmove {}", params),
            EngineMessage::CheckMate(params) => write!(f, "checkmate {}", params),
            EngineMessage::CopyProtection(state) => write!(f, "copyprotection {}", state),
            EngineMessage::Registration(state) => write!(f, "register {}", state),
            EngineMessage::Option(option) => write!(f, "option {}", option),
            EngineMessage::Info(info) => write!(f, "info {}", format_vec!(info)),
            EngineMessage::Unknown(s) => write!(f, "UNKNOWN \"{}\"", s),
        }
    }
}

impl fmt::Display for IdParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            IdParams::Name(name) => write!(f, "name {}", name),
            IdParams::Author(author) => write!(f, "author {}", author),
        }
    }
}

impl fmt::Display for BestMoveParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BestMove { bestmove, ponder } => match ponder {
                Some(ponder) => write!(f, "{} ponder {}", bestmove, ponder),
                _ => write!(f, "{}", bestmove),
            },
            Self::Win => write!(f, "win"),
            Self::Resign => write!(f, "resign"),
        }
    }
}

impl fmt::Display for CheckMateParams {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Mate(mvs) => write!(f, "{}", format_vec!(mvs)),
            Self::NoMate => write!(f, "nomate"),
            Self::TimeOut => write!(f, "timeout"),
            _ => write!(f, "notimplemented"),
        }
    }
}

impl fmt::Display for StatusCheck {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Checking => write!(f, "checking"),
            Self::Ok => write!(f, "ok"),
            Self::Error => write!(f, "error"),
        }
    }
}

impl fmt::Display for OptionParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Check { name, default } => {
                if let Some(default) = default {
                    write!(f, "name {} default {}", name, default)
                } else {
                    write!(f, "name {}", name)
                }
            }
            Self::String { name, default } | Self::Filename { name, default } => match default {
                Some(s) if s.is_empty() => write!(f, "name {} default <empty>", name),
                Some(s) => write!(f, "name {} default {}", name, s),
                _ => write!(f, "name {}", name),
            },
            Self::Button { name } => write!(f, "name {} type button", name),
            Self::Spin {
                name,
                default,
                min,
                max,
            } => {
                let mut opt = format!("name {}", name);
                if let Some(default) = default {
                    opt += &format!(" default {}", default);
                }
                if let Some(min) = min {
                    opt += &format!(" min {}", min);
                }
                if let Some(max) = max {
                    opt += &format!(" max {}", max);
                }
                write!(f, "{}", opt)
            }
            Self::Combo {
                name,
                default,
                vars,
            } => {
                let mut opt = format!("name {}", name);
                if let Some(default) = default {
                    opt += &format!(" default {}", default);
                }
                if !vars.is_empty() {
                    opt += &format!(" var {}", format_vec!(vars, "var "));
                }
                write!(f, "{}", opt)
            }
        }
    }
}

impl fmt::Display for InfoParam {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Depth(n) => write!(f, "depth {}", n),
            Self::SelDepth(n) => write!(f, "seldepth {}", n),
            Self::Time(n) => write!(f, "time {}", n.as_millis()),
            Self::Nodes(n) => write!(f, "nodes {}", n),
            Self::Pv(mvs) => write!(f, "pv {}", format_vec!(mvs)),
            Self::MultiPv(n) => write!(f, "multipv {}", n),
            Self::ScoreCp(cp, bound) => write!(f, "score cp {}{}", cp, bound),
            Self::ScoreMate(plies, bound) => {
                if let Some(plies) = plies {
                    write!(f, "score mate {}{}", plies, bound)
                } else {
                    assert!(*bound == ScoreBound::MateMin || *bound == ScoreBound::MatePlus);
                    write!(f, "score mate{}", bound)
                }
            }
            Self::CurrMove(mv) => write!(f, "currmove {}", mv),
            Self::CurrMoveNum(n) => write!(f, "currmovenum {}", n),
            Self::HashFull(n) => write!(f, "hashfull {}", n),
            Self::Nps(n) => write!(f, "nps {}", n),
            Self::CpuLoad(n) => write!(f, "cpuload {}", n),
            Self::String(s) => write!(f, "string {}", s),
            Self::Refutation(mvs) => write!(f, "refutation {}", format_vec!(mvs)),
            Self::CurrLine { cpu_nr, line } => {
                if let Some(cpu_nr) = cpu_nr {
                    write!(f, "currline cpunr {} {}", cpu_nr, format_vec!(line))
                } else {
                    write!(f, "currline {}", format_vec!(line))
                }
            }
        }
    }
}

impl fmt::Display for ScoreBound {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Lower => write!(f, " lowerbound"),
            Self::Upper => write!(f, " upperbound"),
            Self::MateMin => write!(f, " -"),
            Self::MatePlus => write!(f, " +"),
            _ => write!(f, ""),
        }
    }
}
