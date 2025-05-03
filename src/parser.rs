//! This module implements the USI parser.
//!
//! The [PEG grammar](https://en.wikipedia.org/wiki/Parsing_expression_grammar) used by this crate:
//! ```text
//! WHITESPACE = _{ " " | "\t" }
//! WS = _{ WHITESPACE+ }
//! NL = _{ WHITESPACE | NEWLINE }
//! SEP = _{"/"}
//!
//! // start
//! start = _{SOI ~ line+ ~ &EOI}
//! line = _{ other? ~ (delimited_message | junk) }
//!
//! delimited_message = _{ NL* ~ message ~ WHITESPACE* ~ NEWLINE }
//! message = _{ gui_message | engine_message | other}
//!
//! // other
//! other = @{ (!gui_message ~ !engine_message ~ !NEWLINE ~ ANY)+ }
//! junk = @{ (!NEWLINE ~ ANY)* ~ NEWLINE}
//!
//! // gui message
//! gui_message = _{
//!     usi |
//!     debug |
//!     isready |
//!     setoption |
//!     register |
//!     usinewgame |
//!     stop |
//!     ponderhit |
//!     quit |
//!     position |
//!     go |
//!     gameover
//! }
//!
//! // engine message
//! engine_message = _{
//!     id |
//!     usiok |
//!     readyok |
//!     bestmove |
//!     copyprotection |
//!     registration |
//!     option |
//!     info
//! }
//!
//! //
//! // GuiMessage: GUI to engine
//! //
//!
//! usi = ${ "usi" ~ &NL }
//!
//! debug = ${ "debug" ~ (WS ~ ("on" | "off"))?  }
//!
//! isready = ${ "isready"  }
//!
//! setoption = ${ "setoption" ~ WS ~ "name" ~ WS ~ setoption_name ~ (WS ~ "value" ~ WS ~ setoption_value)? }
//!
//!     setoption_name = ${ !("value") ~ token }
//!     setoption_value = ${ token }
//!
//! register = ${ register_later | register_with_name_and_code }
//!
//!     register_later = { "register" ~ WS ~ "later" }
//!     register_with_name_and_code = { "register" ~ WS ~ "name" ~ WS ~ register_name ~ WS ~ "code" ~ WS ~ register_code }
//!     register_name = { register_token ~ (WS ~ register_token)* }
//!     register_token = _{ !("code") ~ token }
//!     register_code = { tokens }
//!
//! usinewgame = ${ "usinewgame" }
//!
//! stop = ${ "stop"  }
//!
//! quit = ${ "quit"  }
//!
//! ponderhit = ${ "ponderhit"  }
//!
//! gameover = ${ "gameover" ~ WS ~ (win | lose | draw)  }
//!     
//!     win = { "win" }
//!     lose = { "lose" }
//!     draw = { "draw" }
//!
//! position = ${ "position" ~ WS ~ (startpos | sfenpos) ~ (WS ~ "moves" ~ WS ~ moves)?  }
//!
//!     startpos = { "startpos" }
//!     sfenpos = { "sfen" ~ WS ~ sfen_board ~ WS ~ sfen_color ~ WS ~ sfen_hands ~ (WS ~ sfen_move_num)? }
//!
//!     sfen_board = { (sfen_rank ~ SEP){8} ~ sfen_rank }
//!     sfen_color = { "w" | "b" }
//!     sfen_hands = { sfen_black_hand ~ sfen_white_hand? | sfen_white_hand | sfen_empty_hand }
//!     sfen_move_num = { digits }  
//!
//!     sfen_rank = { ((prom? ~ (white_piece | black_piece)) | file){1,9} }
//!
//!     sfen_black_hand = { (npieces? ~ black_hand_piece){1,6} }
//!     sfen_white_hand = { (npieces? ~ white_hand_piece){1,6} }
//!     sfen_empty_hand = { "-" }
//!
//!     prom = { "+" }
//!     white_piece = { "k" | "r"  | "b" | "g" | "s" | "n"  | "l"  | "p" }
//!     black_piece = { "K" | "R"  | "B" | "G" | "S" | "N"  | "L"  | "P" }
//!     white_hand_piece = { "r"  | "b" | "s" | "n"  | "l"  | "p" }
//!     black_hand_piece = { "R"  | "B" | "S" | "N"  | "L"  | "P" }
//!
//!     npieces = { "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" |
//!                 "10" | "11" | "12" | "13" | "14" | "15" | "16" | "17" | "18" }
//!
//!     moves = { one_move ~ (WS ~ one_move)* }
//!
//!     one_move = { drop | board_move }
//!     drop = { black_piece ~ "*" ~ square }
//!     board_move = { square ~ square ~ "+"? }
//!
//!     square = { file ~ rank }
//!     file = { '1'..'9' }
//!     rank = { 'a'..'i' }
//!
//! go = ${ "go" ~ (WS ~ go_sub_cmd)*  }
//!
//!     go_sub_cmd = _{
//!         searchmoves |          
//!         ponder |
//!         movetime |
//!         byoyomi |
//!         movestogo |
//!         wtime |
//!         btime |
//!         winc |
//!         binc |
//!         depth |
//!         nodes |
//!         mate |
//!         infinite
//!     }
//!
//!     searchmoves = { "searchmoves" ~ WS ~ moves}
//!     ponder = { "ponder" }
//!     movetime = { "movetime" ~ WS ~ millisecs }
//!     byoyomi = { "byoyomi" ~ WS ~ millisecs }
//!     wtime = { "wtime" ~ WS ~ millisecs }
//!     btime = { "btime" ~ WS ~ millisecs }
//!     winc = { "winc" ~ WS ~ millisecs }
//!     binc = { "binc" ~ WS ~ millisecs }
//!     movestogo = { "movestogo" ~ WS ~ digits }
//!     depth = { "depth" ~ WS ~ digits }
//!     nodes = { "nodes" ~ WS ~ digits }
//!     // following Shogidokoro semantics
//!     mate = { "mate" ~ WS ~ (infinite | millisecs) }
//!     infinite = { "infinite" }
//!
//! //
//! // EngineMessage: Engine to GUI
//! //
//!
//! id = ${ "id" ~ WS ~ (id_name | id_author) }
//!
//!     id_name = ${ "name" ~ WS ~ tokens }
//!     id_author = ${ "author" ~ WS ~ tokens }
//!
//! usiok = { "usiok"  }
//!
//! readyok = { "readyok"  }
//!
//! bestmove = ${ "bestmove" ~ WS ~ ((one_move ~ (WS ~ ponder_move)?) | resign | win)  }
//!
//!     ponder_move = { "ponder" ~ WS ~ one_move }
//!     resign = { "resign" }
//!
//! copyprotection = ${ "copyprotection" ~ WS ~ status_check  }
//!
//! registration = ${ "registration" ~ WS ~ status_check  }
//!
//!     status_check = { "checking" | "ok" | "error" }
//!
//! option = ${ "option" ~ WS ~ (check_option | spin_option | combo_option | string_option | button_option | filename_option)  }
//!
//!     check_option = ${ option_name ~ WS ~ "type" ~ WS ~ "check" ~ (WS ~ "default" ~ WS ~ check_default)? }
//!     spin_option = ${ option_name ~ WS ~ "type" ~ WS ~ "spin" ~ (WS ~ "default" ~ WS ~ spin_default)? ~ (WS ~ spin_min)? ~ (WS ~ spin_max)? }
//!     combo_option = ${ option_name ~ WS ~ "type" ~ WS ~ "combo" ~ (WS ~ "default" ~ WS ~ default)? ~ (WS ~ "var" ~ WS ~ var)* }
//!     string_option = ${ option_name ~ WS ~ "type" ~ WS ~ "string" ~ (WS ~ "default" ~ WS ~ default)? }
//!     button_option = ${ option_name ~ WS ~ "type" ~ WS ~ "button" }
//!     filename_option = ${ option_name ~ WS ~ "type" ~ WS ~ "filename" ~ (WS ~ "default" ~ WS ~ default)? }
//!
//!     option_name = ${ "name" ~ WS ~ name_token }
//!     name_token = ${ !("type") ~ token }
//!     
//!     check_default = { "true" | "false" }
//!     spin_default = { integer }
//!     spin_min = ${ "min" ~ WS ~ integer }
//!     spin_max = ${ "max" ~ WS ~ integer }
//!     default = ${ "default" ~ WS ~ var_token }
//!     var = { !("var") ~ var_token }
//!     var_token = ${ !("default" | "var") ~ token}
//!     
//! info = ${ "info" ~ (WS ~ info_attr)+  }
//!
//!     info_attr = _{
//!         info_depth |
//!         info_seldepth |
//!         info_time |
//!         info_nodes |
//!         info_currmovenum |
//!         info_currmove |
//!         info_hashfull |
//!         info_nps |
//!         info_cpuload |
//!         info_pv |
//!         info_multipv |
//!         info_refutation |
//!         info_currline |
//!         info_score_cp |
//!         info_score_mate |
//!         info_string
//!     }
//!
//!     info_depth = ${ "depth" ~ WS ~ digits }
//!     info_seldepth = ${ "seldepth" ~ WS ~ digits }
//!     info_time = ${ "time" ~ WS ~ millisecs }
//!     info_nodes = ${ "nodes" ~ WS ~ digits }
//!     info_currmove = ${ "currmove" ~ WS ~ one_move }
//!     info_currmovenum = ${ "currmovenum" ~ WS ~ digits }
//!     info_hashfull = ${ "hashfull" ~ WS ~ digits }
//!     info_nps = ${ "nps" ~ WS ~ digits }
//!     info_cpuload = ${ "cpuload" ~ WS ~ digits }    
//!     info_pv = ${ "pv" ~ WS ~ moves }
//!     info_multipv = ${ "multipv" ~ WS ~ digits }
//!     info_refutation = ${ "refutation" ~ WS ~ moves }
//!     info_currline = ${ "currline" ~ WS ~ (digits ~ WS)? ~ moves }
//!     info_score_cp = ${ "score" ~ WS ~ "cp" ~ WS ~ integer ~ (WS ~ (lowerbound | upperbound))? }
//!     info_score_mate = ${ "score" ~ WS ~ "mate" ~ WS ~ ((integer ~ (WS ~ (lowerbound | upperbound))?) | (plus|minus)) }
//!
//!         lowerbound = { "lowerbound" }
//!         upperbound = { "upperbound" }
//!
//!     info_string = ${ "string" ~ WS ~ tokens }
//!
//! //
//! // helpers
//! //
//!
//! // a token is a contiguous sequence of printable 7-bit ASCII characters (excluding whitespace)
//! token = @{ ('!'..'~')+ }
//!
//! // tokens is a sequence of one or more white-space-separated tokens (excluding newlines)
//! tokens = @{ token ~ (WS ~ token)* }
//!
//! // digits is a contiguous sequence of ascii digits
//! digits = @{ ASCII_DIGIT+ }
//!
//! millisecs = { digits }
//! integer = { (plus | minus)? ~ digits }
//! plus = { "+" }
//! minus = { "-" }
//! ```

#![allow(clippy::result_large_err)]

use core::str::FromStr;
use haitaka_types::Move;
use pest::Parser; // Parser trait
use pest::error::Error as PestError;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser; // Parser proc macro
use std::fmt::Debug;
use std::time::Duration;

use crate::engine::{
    BestMoveParams, EngineMessage, IdParams, InfoParam, OptionParam, ScoreBound, StatusCheck,
};
use crate::gui::{EngineParams, GameStatus, GuiMessage, MateParam};

#[derive(Parser)]
#[grammar = "usi.pest"]
struct UsiParser;

/// This function visualizes the PEST parse tree of any input.
pub fn dbg(s: &str) {
    let res = UsiParser::parse(Rule::start, s);
    if let Ok(pairs) = res {
        println!("{:#?}", pairs);
    } else {
        println!("{:#?}", res);
    }
}

// macros - a few spoonfuls of sugar

/// Extract the string value of a PEST Span as `str`.
/// Also trims leading and trailing whitespace.
macro_rules! as_str {
    ($sp:ident) => {
        $sp.as_span().as_str().trim()
    };
}

/// Extract the string value of a PEST Span as `String`.
/// Also trims leading and trailing whitespace.
macro_rules! as_string {
    ($sp:ident) => {
        $sp.as_span().as_str().trim().to_string()
    };
}

/// Convert "<empty>" into Some(""). Used in parsing `option ... default <empty>`.
macro_rules! convert_empty {
    ($s:ident) => {
        if $s.eq_ignore_ascii_case("<empty>") {
            Some(String::from(""))
        } else {
            Some(String::from($s))
        }
    };
}



impl GuiMessage {
    /// Parse one USI message, sent by the GUI and received by the Engine.
    ///
    /// If the string contains multiple messages, only the first one is returned.
    /// 
    /// Note that all USI protocol messages must be terminated by a newline ('\n', '\r' or '\r\n').
    /// This function will return a ParseError if the input string does not end with either
    /// a newline or newline followed by ascii whitespace.
    ///
    /// SAFETY: The parser should be able to process any newline-terminated input. An input string 
    /// `input` that does not conform to the USI protocol is returned as `Ok(EngineMessage::Unknown(input))`.
    ///
    pub fn parse(input: &str) -> Result<Self, PestError<Rule>> {
        match UsiParser::parse(Rule::start, input) {
            Ok(pairs) => Ok(Self::inner_parse(pairs.into_iter().next().unwrap())),
            Err(err) => Err(err),
        }
    }

    /// Parses the input and returns the first valid protocol GUI message, skipping Unknowns.
    /// Returns `None` if no valid message is found.
    /// 
    /// # Panics
    /// 
    /// This function will panic if the input string is not newline terminated.
    ///
    pub fn parse_first_valid(input: &str) -> Option<Self> {
        GuiMessageStream::new(input).find(|msg| !matches!(msg, GuiMessage::Unknown(_)))
    }

    fn inner_parse(p: Pair<'_, Rule>) -> Self {
        match p.as_rule() {
            Rule::usi => Self::parse_usi(),
            Rule::debug => Self::parse_debug(p),
            Rule::isready => Self::parse_isready(),
            Rule::setoption => Self::parse_setoption(p),
            Rule::register => Self::parse_register(p),
            Rule::usinewgame => Self::parse_usinewgame(),
            Rule::position => Self::parse_position(p),
            Rule::go => Self::parse_go(p),
            Rule::stop => Self::parse_stop(),
            Rule::ponderhit => Self::parse_ponderhit(),
            Rule::gameover => Self::parse_gameover(p),
            Rule::quit => Self::parse_quit(),
            _ => Self::parse_unknown(p.as_str()),
        }
    }

    // unknown
    fn parse_unknown(s: &str) -> Self {
        Self::Unknown(s.to_owned())
    }

    // usi
    fn parse_usi() -> Self {
        Self::Usi
    }

    // debug
    fn parse_debug(pair: Pair<Rule>) -> Self {
        let on = !as_string!(pair).ends_with("off");
        Self::Debug(on)
    }

    // isready
    fn parse_isready() -> Self {
        Self::IsReady
    }

    // setoption
    fn parse_setoption(pair: Pair<Rule>) -> Self {
        let mut name: String = String::default();
        let mut value: Option<String> = None;
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::setoption_name => {
                    name = as_string!(sp);
                }
                Rule::setoption_value => {
                    value = Some(as_string!(sp));
                }
                _ => unreachable!(),
            }
        }
        Self::SetOption { name, value }
    }

    // register
    fn parse_register(pair: Pair<Rule>) -> Self {
        let mut name: Option<String> = None;
        let mut code: Option<String> = None;
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::register_later => {}
                Rule::register_with_name_and_code => {
                    for spi in sp.into_inner() {
                        match spi.as_rule() {
                            Rule::register_name => {
                                name = Some(as_string!(spi));
                            }
                            Rule::register_code => {
                                code = Some(as_string!(spi));
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        Self::Register { name, code }
    }

    // usinewgame
    fn parse_usinewgame() -> Self {
        Self::UsiNewGame
    }

    // position
    fn parse_position(pair: Pair<Rule>) -> Self {
        let mut sfen: Option<String> = None;
        let mut moves: Option<Vec<Move>> = None;
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::startpos => {
                    assert!(sfen.is_none());
                }
                Rule::sfenpos => {
                    sfen = Some(as_str!(sp)
                        .strip_prefix("sfen ")
                        .unwrap()
                        .trim()
                        .to_string());
                }
                Rule::moves => {
                    moves = Some(parse_moves(sp));
                }
                _ => unreachable!(),
            }
        }
        Self::Position { sfen, moves }
    }

    // go
    fn parse_go(pair: Pair<Rule>) -> Self {
        let mut params = EngineParams::new();

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::searchmoves => {
                    params = params.searchmoves(parse_moves(sp));
                }
                Rule::depth => {
                    params = params.depth(parse_digits::<u16>(sp));
                }
                Rule::nodes => {
                    params = params.nodes(parse_digits::<u32>(sp));
                }
                Rule::mate => {
                    for spi in sp.into_inner() {
                        match spi.as_rule() {
                            Rule::millisecs => {
                                params = params.mate(MateParam::Timeout(parse_millisecs(spi)))
                            }
                            Rule::infinite => params = params.mate(MateParam::Infinite),
                            _ => unreachable!(),
                        }
                    }
                }
                Rule::byoyomi => {
                    params = params.byoyomi(parse_millisecs(sp));
                }
                Rule::btime => {
                    params = params.btime(parse_millisecs(sp));
                }
                Rule::wtime => {
                    params = params.wtime(parse_millisecs(sp));
                }
                Rule::binc => {
                    params = params.binc(parse_millisecs(sp));
                }
                Rule::winc => {
                    params = params.winc(parse_millisecs(sp));
                }
                Rule::movestogo => {
                    params = params.movestogo(parse_digits::<u16>(sp));
                }

                // implicit assumption is that these are alternatives   TODO: double-check
                Rule::ponder => {
                    params = params.ponder();
                }
                Rule::movetime => params = params.movetime(parse_millisecs(sp)),
                Rule::infinite => {
                    params = params.infinite();
                }
                _ => unreachable!(),
            }
        }
        Self::Go(params)
    }

    // stop
    fn parse_stop() -> Self {
        Self::Stop
    }

    // ponderhit
    fn parse_ponderhit() -> Self {
        Self::PonderHit
    }

    // gameover
    fn parse_gameover(pair: Pair<Rule>) -> Self {
        if let Some(sp) = pair.into_inner().next() {
            match sp.as_rule() {
                Rule::win => return Self::GameOver(GameStatus::Win),
                Rule::lose => return Self::GameOver(GameStatus::Lose),
                Rule::draw => return Self::GameOver(GameStatus::Draw),
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    // quit
    fn parse_quit() -> Self {
        Self::Quit
    }
}

/// The GuiMessageStream struct enables iteration over a multi-line text string.
pub struct GuiMessageStream<'a> {
    /// Inner PEST iterator over grammar Rules
    pairs: Pairs<'a, Rule>,
}

impl<'a> GuiMessageStream<'a> {
    /// Create a new `GuiMessageStream` from an input string.
    ///
    /// SAFETY: Since the grammar is designed to process any input, this should never fail.
    pub fn new(input: &'a str) -> Self {
        Self::parse(input)
    }

    /// Parse a multi-line input string and return a GuiMessageStream instance.
    ///
    /// SAFETY: Since the parser should be able to handle any input, this should never fail.
    pub fn parse(input: &'a str) -> Self {
        Self::try_parse(input).expect("Internal error: Failed to initialize UsiParser.")
    }

    pub fn try_parse(input: &'a str) -> Result<Self, PestError<Rule>> {
        let pairs = UsiParser::parse(Rule::start, input);
        match pairs {
            Ok(pairs) => Ok(Self { pairs }),
            Err(err) => Err(err),
        }
    }
}

impl Iterator for GuiMessageStream<'_> {
    type Item = GuiMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pair) = self.pairs.by_ref().next() {
            let res = GuiMessage::inner_parse(pair);
            return Some(res);
        }
        None
    }
}

// EngineMessage parser

impl EngineMessage {
    /// Parse one USI message, sent by the Engine and received by the GUI.
    ///
    /// If the string contains multiple messages, only the first one is returned.
    ///
    /// Note that all USI protocol messages must be terminated by a newline ('\n', '\r' or '\r\n').
    /// This function will return a ParseError if the input string does not end with either
    /// a newline or newline followed by ascii whitespace.
    ///
    /// SAFETY: The parser should be able to process any newline-terminated input. An input string `input` 
    /// that does not conform to the USI protocol is returned as `Ok(GuiMessage::Unknown(input))`.
    ///
    pub fn parse(input: &str) -> Result<Self, PestError<Rule>> {
        match UsiParser::parse(Rule::start, input) {
            Ok(pairs) => Ok(Self::inner_parse(pairs.into_iter().next().unwrap())),
            Err(err) => Err(err),
        }
    }

    /// Parses the input and returns the first valid protocol Engine message, skipping Unknowns.
    /// Returns `None` if no valid Engine message is found.
    /// 
    /// # Panics
    /// 
    /// This function will panic if the input string is not newline terminated.
    /// 
    pub fn parse_first_valid(input: &str) -> Option<Self> {
        EngineMessageStream::new(input).find(|msg| !matches!(msg, EngineMessage::Unknown(_)))
    }

    pub fn inner_parse(p: Pair<'_, Rule>) -> Self {
        match p.as_rule() {
            Rule::id => Self::parse_id(p),
            Rule::usiok => Self::parse_usiok(),
            Rule::readyok => Self::parse_readyok(),
            Rule::bestmove => Self::parse_bestmove(p),
            Rule::copyprotection => Self::parse_copyprotection(p),
            Rule::registration => Self::parse_registration(p),
            Rule::option => Self::parse_option(p),
            Rule::info => Self::parse_info(p),
            _ => Self::parse_unknown(p.as_str()),
        }
    }

    // unknown
    fn parse_unknown(s: &str) -> Self {
        Self::Unknown(s.to_owned())
    }

    // id
    fn parse_id(pair: Pair<Rule>) -> Self {
        if let Some(sp) = pair.into_inner().next() {
            match sp.as_rule() {
                Rule::id_name => return EngineMessage::Id(IdParams::Name(parse_tokens(sp))),
                Rule::id_author => return EngineMessage::Id(IdParams::Author(parse_tokens(sp))),
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    // usiok
    fn parse_usiok() -> Self {
        EngineMessage::UsiOk
    }

    // readyok
    fn parse_readyok() -> Self {
        EngineMessage::ReadyOk
    }

    // bestmove
    fn parse_bestmove(pair: Pair<Rule>) -> Self {
        let mut bestmove: Option<Move> = None;
        let mut ponder: Option<Move> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::one_move => {
                    bestmove = Some(Move::from_str(as_str!(sp)).unwrap());
                }
                Rule::ponder_move => {
                    ponder = Some(Move::from_str(as_str!(sp)).unwrap());
                }
                Rule::resign => return EngineMessage::BestMove(BestMoveParams::Resign),
                Rule::win => return EngineMessage::BestMove(BestMoveParams::Win),
                _ => unreachable!(),
            }
        }

        if let Some(bestmove) = bestmove {
            EngineMessage::BestMove(BestMoveParams::BestMove { bestmove, ponder })
        } else {
            unreachable!()
        }
    }

    // copyprotection
    fn parse_copyprotection(pair: Pair<Rule>) -> Self {
        let state = Self::parse_status_check(pair);
        EngineMessage::CopyProtection(state)
    }

    // registration
    fn parse_registration(pair: Pair<Rule>) -> Self {
        let state = Self::parse_status_check(pair);
        EngineMessage::Registration(state)
    }

    fn parse_status_check(pair: Pair<Rule>) -> StatusCheck {
        for sp in pair.into_inner() {
            if let Rule::status_check = sp.as_rule() {
                let s = as_str!(sp);
                match s {
                    "checking" => return StatusCheck::Checking,
                    "ok" => return StatusCheck::Ok,
                    "error" => return StatusCheck::Error,
                    _ => unreachable!(),
                };
            }
        }
        unreachable!()
    }

    // option
    fn parse_option(pair: Pair<Rule>) -> Self {
        if let Some(sp) = pair.into_inner().next() {
            match sp.as_rule() {
                Rule::check_option => return Self::parse_check_option(sp),
                Rule::spin_option => return Self::parse_spin_option(sp),
                Rule::combo_option => return Self::parse_combo_option(sp),
                Rule::string_option => return Self::parse_string_option(sp),
                Rule::button_option => return Self::parse_button_option(sp),
                Rule::filename_option => return Self::parse_filename_option(sp),
                _ => unreachable!(),
            }
        }
        unreachable!()
    }

    // options name ... type check ...
    fn parse_check_option(pair: Pair<Rule>) -> Self {
        let mut name: Option<String> = None;
        let mut default: Option<bool> = None;
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::option_name => name = Some(parse_tokens(sp)),
                Rule::check_default => default = Some(as_string!(sp).eq_ignore_ascii_case("true")),
                _ => (),
            }
        }
        if let Some(name) = name {
            Self::Option(OptionParam::Check { name, default })
        } else {
            unreachable!()
        }
    }

    // option name ... type spin ...
    fn parse_spin_option(pair: Pair<Rule>) -> Self {
        let mut name: Option<String> = None;
        let mut default: Option<i32> = None;
        let mut min: Option<i32> = None;
        let mut max: Option<i32> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::option_name => name = Some(parse_tokens(sp)),
                Rule::spin_default => default = Some(parse_integer::<i32>(sp)),
                Rule::spin_min => min = Some(parse_integer::<i32>(sp)),
                Rule::spin_max => max = Some(parse_integer::<i32>(sp)),
                _ => (),
            }
        }

        if let Some(name) = name {
            Self::Option(OptionParam::Spin {
                name,
                default,
                min,
                max,
            })
        } else {
            unreachable!()
        }
    }

    // option name ... type combo ...
    fn parse_combo_option(pair: Pair<Rule>) -> Self {
        let mut name: Option<String> = None;
        let mut default: Option<String> = None;
        let mut vars: Vec<String> = Vec::new();

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::option_name => name = Some(parse_tokens(sp)),
                Rule::combo_default => default = Some(parse_tokens(sp)),
                Rule::var_token => vars.push(parse_tokens(sp)),
                _ => (),
            }
        }

        if let Some(name) = name {
            Self::Option(OptionParam::Combo {
                name,
                default,
                vars,
            })
        } else {
            unreachable!()
        }
    }

    // option name ... type string
    fn parse_string_option(pair: Pair<Rule>) -> Self {
        let mut name: Option<String> = None;
        let mut default: Option<String> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::option_name => name = Some(parse_tokens(sp)),
                Rule::token => {
                    default = {
                        let s = as_string!(sp);
                        convert_empty!(s)
                    }
                }
                _ => (),
            }
        }

        if let Some(name) = name {
            Self::Option(OptionParam::String { name, default })
        } else {
            unreachable!()
        }
    }

    // option name ... type button
    fn parse_button_option(pair: Pair<Rule>) -> Self {
        for sp in pair.into_inner() {
            if sp.as_rule() == Rule::option_name {
                let name = parse_tokens(sp);
                return Self::Option(OptionParam::Button { name });
            }
        }
        unreachable!()
    }

    // option name ... type filename ...
    fn parse_filename_option(pair: Pair<Rule>) -> Self {
        let mut name: Option<String> = None;
        let mut default: Option<String> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::option_name => name = Some(parse_tokens(sp)),
                Rule::token => {
                    default = {
                        let s = as_string!(sp);
                        convert_empty!(s)
                    }
                }
                _ => (),
            }
        }

        if let Some(name) = name {
            Self::Option(OptionParam::Filename { name, default })
        } else {
            unreachable!()
        }
    }

    // info
    fn parse_info(pair: Pair<Rule>) -> Self {
        let mut v: Vec<InfoParam> = Vec::<InfoParam>::new();
        for sp in pair.into_inner() {
            let info: InfoParam = match sp.as_rule() {
                Rule::info_depth => InfoParam::Depth(parse_digits::<u16>(sp)),
                Rule::info_seldepth => InfoParam::SelDepth(parse_digits::<u16>(sp)),
                Rule::info_time => InfoParam::Time(parse_millisecs(sp)),
                Rule::info_nodes => InfoParam::Nodes(parse_digits::<u64>(sp)),
                Rule::info_currmovenum => InfoParam::CurrMoveNum(parse_digits::<u16>(sp)),
                Rule::info_currmove => InfoParam::CurrMove(parse_move(sp)),
                Rule::info_hashfull => InfoParam::HashFull(parse_digits::<u16>(sp)),
                Rule::info_nps => InfoParam::Nps(parse_digits::<u64>(sp)),
                Rule::info_cpuload => InfoParam::CpuLoad(parse_digits::<u16>(sp)),
                Rule::info_multipv => InfoParam::MultiPv(parse_digits::<u16>(sp)),
                Rule::info_string => InfoParam::String(as_string!(sp)),
                Rule::info_pv => InfoParam::Pv(parse_moves(sp)),
                Rule::info_refutation => InfoParam::Refutation(parse_moves(sp)),
                Rule::info_currline => Self::parse_currline(sp),
                Rule::info_score_cp => Self::parse_score_cp(sp),
                Rule::info_score_mate => Self::parse_score_mate(sp),
                _ => unreachable!(),
            };
            v.push(info);
        }
        EngineMessage::Info(v)
    }

    // info currline ...
    fn parse_currline(pair: Pair<Rule>) -> InfoParam {
        let mut cpu_nr: Option<u16> = None;
        let mut line: Vec<Move> = Vec::<Move>::new();

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::digits => cpu_nr = Some(parse_digits::<u16>(sp)),
                Rule::moves => line = parse_moves(sp),
                _ => unreachable!(),
            }
        }
        InfoParam::CurrLine { cpu_nr, line }
    }

    // info score cp ...
    fn parse_score_cp(pair: Pair<Rule>) -> InfoParam {
        let mut v: Option<i32> = None;
        let mut bound: ScoreBound = ScoreBound::Exact;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::integer => {
                    let s = as_str!(sp); // Extract the string representation
                    v = Some(s.parse::<i32>().unwrap_or_else(|err| {
                        unreachable!(
                            "PEST grammar bug: failed to parse integer '{}': {:?}",
                            s, err
                        )
                    }));
                }
                Rule::lowerbound => bound = ScoreBound::Lower,
                Rule::upperbound => bound = ScoreBound::Upper,
                _ => unreachable!(), // really?
            }
        }

        if let Some(value) = v {
            InfoParam::ScoreCp(value, bound)
        } else {
            unreachable!()
        }
    }

    // info score mate ...
    fn parse_score_mate(pair: Pair<Rule>) -> InfoParam {
        let mut v: Option<i32> = None;
        let mut bound: ScoreBound = ScoreBound::Exact;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::integer => {
                    let s = as_str!(sp); // Extract the string representation
                    v = Some(s.parse::<i32>().unwrap());
                }
                Rule::plus => bound = ScoreBound::MatePlus,
                Rule::minus => bound = ScoreBound::MateMin,
                Rule::lowerbound => bound = ScoreBound::Lower,
                Rule::upperbound => bound = ScoreBound::Upper,
                _ => unreachable!(),
            }
        }
        InfoParam::ScoreMate(v, bound)
    }
}

/// The EngineMessageStream struct enables iteration over a multi-line text string.
pub struct EngineMessageStream<'a> {
    /// Inner PEST iterator over grammar Rules
    pairs: Pairs<'a, Rule>,
}

impl<'a> EngineMessageStream<'a> {
    /// Create a new `EngineMessageStream` from an input string.
    ///
    /// SAFETY: Since the grammar is designed to process any input, this should never fail.
    pub fn new(input: &'a str) -> Self {
        Self::parse(input)
    }

    /// Parse an input string and return a new `EngineMessageStream`.
    ///
    /// SAFETY: Since the grammar is designed to process any input, this should never fail.
    pub fn parse(input: &'a str) -> Self {
        Self::try_parse(input).expect("Internal error: Failed to initialize UsiParser.")
    }

    pub fn try_parse(input: &'a str) -> Result<Self, PestError<Rule>> {
        let pairs = UsiParser::parse(Rule::start, input);
        match pairs {
            Ok(pairs) => Ok(Self { pairs }),
            Err(err) => Err(err),
        }
    }
}

impl Iterator for EngineMessageStream<'_> {
    type Item = EngineMessage;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pair) = self.pairs.by_ref().next() {
            let res = EngineMessage::inner_parse(pair);
            return Some(res);
        }
        None
    }
}

// HELPERS

// SAFETY: The PEST grammar ensures that all low-level parse/unwrap calls are safe.
// Panics are justified since any panic would indicate a serious bug either in the
// way this module hooks up the functions to the grammar or in the grammar itself.

fn parse_move(pair: Pair<Rule>) -> Move {
    for sp in pair.into_inner() {
        if let Rule::one_move = sp.as_rule() {
            return as_str!(sp).parse::<Move>().unwrap();
        }
    }
    unreachable!()
}

fn parse_moves(pair: Pair<Rule>) -> Vec<Move> {
    let mut moves = Vec::<Move>::new();

    for sp in pair.into_inner() {
        match sp.as_rule() {
            Rule::one_move => {
                let mv = Move::from_str(as_str!(sp)).unwrap();
                moves.push(mv);
            }
            Rule::moves => {
                let mvs: Vec<Move> = parse_moves(sp);
                moves.extend(mvs);
            }
            _ => unreachable!(),
        }
    }

    moves
}

fn parse_digits<T>(pair: Pair<Rule>) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    for sp in pair.into_inner() {
        if let Rule::digits = sp.as_rule() {
            return as_str!(sp).parse::<T>().unwrap();
        }
    }
    unreachable!()
}

fn parse_integer<T>(pair: Pair<Rule>) -> T
where
    T: FromStr,
    T::Err: Debug,
{
    for sp in pair.into_inner() {
        if let Rule::integer = sp.as_rule() {
            return as_str!(sp).parse::<T>().unwrap();
        }
    }
    unreachable!()
}

fn parse_millisecs(pair: Pair<Rule>) -> Duration {
    for sp in pair.into_inner() {
        if let Rule::millisecs = sp.as_rule() {
            let milliseconds: u64 = as_str!(sp).parse::<u64>().unwrap();
            return Duration::from_millis(milliseconds);
        }
        if let Rule::digits = sp.as_rule() {
            let milliseconds: u64 = as_str!(sp).parse::<u64>().unwrap();
            return Duration::from_millis(milliseconds);
        }
    }
    unreachable!()
}

fn parse_tokens(pair: Pair<'_, Rule>) -> String {
    for sp in pair.into_inner() {
        match sp.as_rule() {
            Rule::tokens => return as_string!(sp).to_owned(),
            Rule::token => return as_string!(sp).to_owned(),
            _ => return parse_tokens(sp),
        }
    }
    unreachable!()
}
