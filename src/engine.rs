//! This module contains the data model for EngineMessage, the commands sent from Engine to GUI.
//! The main enum is [`EngineMessage`] which encodes all messages sent from a Shogi engine to the GUI.
//!
//! For full documenation about the protocol see
//! - [将棋所USIプロトコル](https://shogidokoro2.stars.ne.jp/usi.html)
//! - [The Universal Shogi Interface](http://hgm.nubati.net/usi.html)
use crate::format_vec;
use haitaka_types::Move;
use std::fmt;
use std::time::Duration;

/// Messages sent from the Shogi Engine to the GUI.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum EngineMessage {
    /// `id` - the `id` message informs the GUI about the engine name and engine
    /// developer. This message is sent as initial response to the GUI `usi` message.
    /// ```text
    /// id name haitaka-shogi
    /// id author tofutofu
    /// ```
    Id(IdParams),

    /// `usiok` - sent to finalize the initial handshake between GUI and engine. This
    /// message is sent after the `id` and initial `option` messages.
    UsiOk,

    /// `readyok` - sent in response to the `isready` message to inform the GUI that
    /// the engine is ready to start a search.
    ReadyOk,

    /// `bestmove` - sent in response to a `go` command, to inform the GUI that the engine
    /// has stopped searching and found a good move. The engine can also use this command
    /// to resign or claim a win.
    /// ```text
    /// bestmove <move>
    /// bestmove <move> ponder <move>
    /// bestmove resign
    /// bestmove win
    /// ```
    BestMove(BestMoveParams),

    /// `checkmate` - sent as termination of a `go mate` command.
    /// ```text
    /// checkmate <moves> - a forced mate principal sequence of moves (sokuzumi)
    /// checkmate nomate - the engine was able to prove there is no forced mate
    /// checkmate timeout - the search timed out inconclusively
    /// checkmate notimplemented - the engine does not implement tsume shogi search
    /// ```
    CheckMate(CheckMateParams),

    /// `copyprotection` - sent by engines that check copy protection:
    /// ```text
    /// copyprotection error
    /// copyprotection checking
    /// copyprotection ok
    /// ```
    CopyProtection(StatusCheck),

    /// `registration` - sent by engines that may require the user (GUI) to register
    /// by name and registration code.
    /// ```text
    /// registration error
    /// registration checking
    /// registration ok
    /// ```
    Registration(StatusCheck),

    /// `option` - informs the GUI about available engine options. Options are
    /// distinguished by type and name. Examples:
    /// ```text
    /// option name UseBook type check default true
    /// option name Selectivity type spin default 2 min 0 max 4
    /// option name Style type combo default Normal var Solid var Normal var Risky
    /// option name ResetLearning type button
    /// option name BookFile type string default public.bin
    /// option name LearningFile type filename default <empty>
    /// ```
    /// Available engine options are sent by the engine in response to the `usi` command.
    /// The GUI can in that case respond by modifying an option with a `setoption` message.
    /// Option names can never have spaces. Certain names have fixed semantics:
    /// ```text
    /// - USI_Hash type spin - MB memory to use for hash tables
    /// - USI_Ponder type check - when set, the engine is allowed to "ponder" (think during opponent's time)
    /// - USI_OwnBook type check - the engine has its own opening book
    /// - USI_Multipv type spin - the engine supports multi bestline mode (default is 1)
    /// - USI_ShowCurrLine type check - the engine can show the current line while searching (false by default)
    /// - USI_ShowRefutations type check - the engine can show a move and its refutation (false by default)
    /// - USI_LimitStrength type check - the engine can adjust its strength (false by default)
    /// - USI_Strength type spin - the engine plays at the indicated strenght level (negative values
    /// represent kyu levels, positive values dan levels), requires USI_LimitStrength to be set
    /// - USI_AnalyseMode type check - the engine may behave differently when analysing or playing a game
    /// ```
    /// The `USI_Hash` and especially the `USI_Ponder` options should always be supported. Note that even
    /// when the ponder option is available and enabled, the engine should still only start pondering when
    /// it receives a `go ponder` command.
    ///
    /// Engines may only support a subset of options. For details, please consult the engine documentation.
    Option(OptionParam),

    /// `info` - informs the GUI, during the search, about the status of the search. The engine may send
    /// either selected `info` messages or multiple infos in one message. All infos about the principal
    /// variation should be sent in one message. Infos about multipv should be sent in successive
    /// lines. Examples:
    /// ```text
    /// info time 1141 depth 3 nodes 135125 score cp -1521 pv 3a3b L*4h 4c4d
    /// info depth 2 score cp 214 time 1242 nodes 2124 nps 34928 pv 2g2f 8c8d 2f2e
    /// info nodes 120000 nps 116391 hashfull 104
    /// info string 7g7f (70%)
    /// info score cp 156 multipv 1 pv P*5h 4g5g 5h5g 8b8f
    /// info score cp -99 multipv 2 pv 2d4d 3c4e 8h5e N*7f
    /// info score cp -157 multipv 3 pv 5g5f 4g4f 4e3c+ 4c3c
    /// ```
    Info(Vec<InfoParam>),

    /// This variant is a catch-all for messages that do not conform to the USI protocol.
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

    /// Signifies error in copy protection or registration validation.
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
    /// The `info depth` message. Search depth in plies.
    Depth(u16),

    /// The `info seldepth` message. Selective search depth in plies.
    /// This also requires `depth` to be sent.
    SelDepth(u16),

    /// The `info time` message. The time searched. Should be sent with the pv.
    Time(Duration),

    /// The `info nodes` message. Number of nodes searched.
    Nodes(u64),

    /// The `info pv` message (principal variation, best line).
    Pv(Vec<Move>),

    /// The `info pv ... multipv` message (the pv line number in a multi pv sequence).
    MultiPv(u16),

    /// The 'info score cp ...' message. The score in centipawns (from engine's point of view).
    ScoreCp(i32, ScoreBound),

    /// The info score mate ...' message. Mate in this many plies. Negative values are
    /// used to indicate that the engine is being mated.
    ScoreMate(Option<i32>, ScoreBound),

    /// The `info currmove` message (current move being searched).
    CurrMove(Move),

    /// The `info currmovenum` message (current move number).
    CurrMoveNumber(u16),

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

// Note that the Display for EngineMessage does not add a terminating newline character.
// When actually sending protocol messages a writer should add the '\n'.

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
                    write!(f, "name {} type check default {}", name, default)
                } else {
                    write!(f, "name {} type check", name)
                }
            }
            Self::String { name, default } => match default {
                Some(s) if s.is_empty() => write!(f, "name {} type string default <empty>", name),
                Some(s) => write!(f, "name {} type string default {}", name, s),
                _ => write!(f, "name {} type string", name), // seems invalid
            },
            Self::Filename { name, default } => match default {
                Some(s) if s.is_empty() => write!(f, "name {} type filename default <empty>", name),
                Some(s) => write!(f, "name {} type filename default {}", name, s),
                _ => write!(f, "name {} type filename", name), // seems invalid
            },
            Self::Button { name } => write!(f, "name {} type button", name),
            Self::Spin {
                name,
                default,
                min,
                max,
            } => {
                let mut opt = format!("name {} type spin", name);
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
                let mut opt = format!("name {} type combo", name);
                if let Some(default) = default {
                    opt += &format!(" default {}", default);
                }
                if !vars.is_empty() {
                    opt += &format!(" var {}", format_vec!(vars, " var "));
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
                    debug_assert!(*bound == ScoreBound::MateMin || *bound == ScoreBound::MatePlus);
                    write!(f, "score mate{}", bound)
                }
            }
            Self::CurrMove(mv) => write!(f, "currmove {}", mv),
            Self::CurrMoveNumber(n) => write!(f, "currmovenumber {}", n),
            Self::HashFull(n) => write!(f, "hashfull {}", n),
            Self::Nps(n) => write!(f, "nps {}", n),
            Self::CpuLoad(n) => write!(f, "cpuload {}", n),
            Self::String(s) => write!(f, "string {}", s),
            Self::Refutation(mvs) => write!(f, "refutation {}", format_vec!(mvs)),
            Self::CurrLine { cpu_nr, line } => {
                if let Some(cpu_nr) = cpu_nr {
                    write!(f, "currline {} {}", cpu_nr, format_vec!(line))
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
