//! This module contains the data-model for GuiMessage, the commands sent from the GUI to the Engine.
use chrono::Duration;
use std::fmt;
// use std::time::Duration;
use crate::format_vec;
use haitaka_types::Move;

pub const SFEN_STARTPOS: &str = "lnsgkgsnl/1r5b1/ppppppppp/9/9/9/PPPPPPPPP/1B5R1/LNSGKGSNL b - 1";

/// Messages sent from the GUI to the engine.
///
/// All variant names correspond to the USI messages.
#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub enum GuiMessage {
    Usi,
    Debug(bool),
    IsReady,
    SetOption {
        name: String,
        value: Option<String>,
    },
    Register {
        name: Option<String>,
        code: Option<String>,
    },
    UsiNewGame,
    Position {
        sfen: Option<String>,
        moves: Option<Vec<Move>>,
    },
    Go(EngineParams),
    Stop,
    PonderHit,
    GameOver(GameStatus),
    Quit,
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
    /// Find a mate in this many ms
    Timeout(Duration),
    /// Search indefinitely long until finding a forced mate
    Infinite,
}

/*
impl Default for EngineParams {
    fn default() -> Self {
        EngineParams {
            searchmoves: None, // No restriction on search moves by default
            ponder: false,     // Ponder mode is off by default
            btime: None,       // No black time left specified
            wtime: None,       // No white time left specified
            binc: None,        // No black increment specified
            winc: None,        // No white increment specified
            byoyomi: None,     // No byoyomi time specified
            movestogo: None,   // No moves-to-go specified
            depth: None,       // No depth limit specified
            nodes: None,       // No node limit specified
            mate: None,        // No mate search parameters specified
            movetime: None,    // No specific move time specified
            infinite: false,   // Infinite search is off by default
        }
    }
}
*/

/// EngineParams initialization.
///
/// # Examples
///
/// ```
/// use haitaka_usi::gui::*;
/// let params1 = EngineParams::new().ponder().infinite();
/// let params2 = EngineParams {
///     ponder: true,
///     infinite: true,
///     ..Default::default()
/// };
/// assert_eq!(params1, params2);
/// ```
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
            GuiMessage::Unknown(s) => write!(f, "UNKNOWN {}", s),
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
            params += &format!(" btime {}", btime.num_milliseconds());
        }
        if let Some(wtime) = self.wtime {
            params += &format!(" wtime {}", wtime.num_milliseconds());
        }
        if let Some(binc) = self.binc {
            params += &format!(" binc {}", binc.num_milliseconds());
        }
        if let Some(winc) = self.winc {
            params += &format!(" winc {}", winc.num_milliseconds());
        }
        if let Some(byoyomi) = self.byoyomi {
            params += &format!(" byoyomi {}", byoyomi.num_milliseconds());
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
                    params += &format!(" mate {}", duration.num_milliseconds());
                }
                MateParam::Infinite => {
                    params += " mate infinite";
                }
            }
        }
        if let Some(movetime) = self.movetime {
            params += &format!(" movetime {}", movetime.num_milliseconds());
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
