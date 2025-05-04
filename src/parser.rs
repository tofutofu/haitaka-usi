//! This module implements the USI parser.
//!
//! The [PEG grammar](https://en.wikipedia.org/wiki/Parsing_expression_grammar) used by this crate
//! is part of the source code as [usi.pest](https://github.com/tofutofu/haitaka-usi/blob/main/src/usi.pest)
//!
//! The main parse functions are
//! - [`GuiMessage::parse`]
//! - [`GuiMessage::parse_first_valid`]
//! - [`EngineMessage::parse`]
//! - [`EngineMessage::parse_first_valid`]
//!
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

/// Extract a Move from a PEST Pair.
macro_rules! as_move {
    ($sp:ident) => {
        Move::from_str(as_str!($sp)).unwrap()
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
    /// # Examples
    ///
    /// ```
    /// use haitaka_usi::*;
    /// let input = "usi\n";
    /// let msg = GuiMessage::parse(input).unwrap();
    /// assert_eq!(msg, GuiMessage::Usi);
    /// ```
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
    /// # Examples
    ///
    /// ```
    /// use haitaka_usi::*;
    /// let input = "yo\nyo usinewgame\n";
    /// let msg = GuiMessage::parse_first_valid(input).unwrap();
    /// assert_eq!(msg, GuiMessage::UsiNewGame);
    /// ```
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
                    sfen = Some(
                        as_str!(sp)
                            .strip_prefix("sfen ")
                            .unwrap()
                            .trim()
                            .to_string(),
                    );
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
    /// # Examples
    ///
    /// ```
    /// use haitaka_usi::*;
    /// use haitaka_types::*;
    /// let input = "bestmove 3c3d\n";
    /// let msg = EngineMessage::parse(input).unwrap();
    /// let mv = "3c3d".parse::<Move>().unwrap();
    /// assert_eq!(msg,
    ///     EngineMessage::BestMove(
    ///         BestMoveParams::BestMove {
    ///             bestmove: mv,
    ///             ponder: None })
    /// );
    /// let input = "bestmove resign\n";
    /// let msg = EngineMessage::parse(input).unwrap();
    /// assert_eq!(msg,
    ///     EngineMessage::BestMove(
    ///         BestMoveParams::Resign
    ///     )
    /// );
    /// ```
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

    fn inner_parse(p: Pair<'_, Rule>) -> Self {
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
                    bestmove = Some(as_move!(sp));
                }
                Rule::ponder_move => {
                    ponder = Some(parse_move(sp));
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
                Rule::info_currmovenumber => InfoParam::CurrMoveNumber(parse_digits::<u16>(sp)),
                Rule::info_currmove => InfoParam::CurrMove(parse_move(sp)),
                Rule::info_hashfull => InfoParam::HashFull(parse_digits::<u16>(sp)),
                Rule::info_nps => InfoParam::Nps(parse_digits::<u64>(sp)),
                Rule::info_cpuload => InfoParam::CpuLoad(parse_digits::<u16>(sp)),
                Rule::info_multipv => InfoParam::MultiPv(parse_digits::<u16>(sp)),
                Rule::info_string => InfoParam::String(parse_tokens(sp)),
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
                Rule::cpunr => cpu_nr = Some(parse_digits::<u16>(sp)),
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
            return as_move!(sp);
        }
    }
    unreachable!()
}

fn parse_moves(pair: Pair<Rule>) -> Vec<Move> {
    let mut moves = Vec::<Move>::new();

    for sp in pair.into_inner() {
        match sp.as_rule() {
            Rule::one_move => {
                moves.push(as_move!(sp));
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
    if let Some(sp) = pair.into_inner().next() {
        match sp.as_rule() {
            Rule::tokens => return as_string!(sp).to_owned(),
            Rule::token => return as_string!(sp).to_owned(),
            _ => return parse_tokens(sp),
        }
    }
    unreachable!()
}
