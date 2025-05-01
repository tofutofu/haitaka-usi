// parser.rs
#![allow(clippy::result_large_err)]

use chrono::Duration;
use std::num::ParseIntError;
// use std::time::Duration;
use core::str::FromStr;
use haitaka_types::{Move, MoveParseError};
use pest::Parser; // trait
use pest::error::Error as PestError;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser; // proc macro
use std::fmt::Debug;

use crate::engine::{
    BestMoveParams, EngineMessage, IdParams, InfoParam, OptionParam, ScoreBound, StatusCheck,
};
use crate::gui::{EngineParams, GameStatus, GuiMessage};

#[derive(Parser)]
#[grammar = "usi.pest"]
struct UsiParser;

pub fn dbg(s: &str) {
    let res = UsiParser::parse(Rule::start, s);
    if let Ok(pairs) = res {
        println!("{:#?}", pairs);
    } else {
        println!("{:#?}", res);
    }
}

// macros
//
// ```
//    Just a spoonful of sugar helps the medicine go down
//    In a most delightful way
// ```

// Extract the string value of a PEST Span as `str`.
// Also trims leading and trailing whitespace.
macro_rules! as_str {
    ($sp:ident) => {
        $sp.as_span().as_str().trim()
    };
}

// Extract the string value of a PEST Span as `String`.
// Also trims leading and trailing whitespace.
macro_rules! as_string {
    ($sp:ident) => {
        $sp.as_span().as_str().trim().to_string()
    };
}

// Convert "<empty>" into Some(""). Used in parsing `option ... default <empty>`.
macro_rules! convert_empty {
    ($s:ident) => {
        if $s.eq_ignore_ascii_case("<empty>") {
            Some(String::from(""))
        } else {
            Some(String::from($s))
        }
    };
}

use thiserror::Error;

#[allow(clippy::large_enum_variant)]
#[derive(Error, Debug)]
pub enum ParseError {
    #[error("pest: {0}")]
    PestError(#[from] PestError<Rule>),

    #[error("move: {0}")]
    ParseMoveError(#[from] MoveParseError),

    #[error("int: {0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("invalid USI syntax")]
    SyntaxError,
}

impl GuiMessage {
    /// Parse a USI message from Gui to Engine.
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let pairs: Pairs<Rule> = UsiParser::parse(Rule::start, s)?;

        if let Some(p) = pairs.into_iter().next() {
            let msg: Self = match p.as_rule() {
                Rule::usi => Self::parse_usi()?,
                Rule::debug => Self::parse_debug(p)?,
                Rule::isready => Self::parse_isready()?,
                Rule::setoption => Self::parse_setoption(p)?,
                Rule::register => Self::parse_register(p)?,
                Rule::usinewgame => Self::parse_usinewgame()?,
                Rule::position => Self::parse_position(p)?,
                Rule::go => Self::parse_go(p)?,
                Rule::stop => Self::parse_stop()?,
                Rule::ponderhit => Self::parse_ponderhit()?,
                Rule::gameover => Self::parse_gameover(p)?,
                Rule::quit => Self::parse_quit()?,
                _ => Self::parse_unknown(p.as_str()),
            };
            return Ok(msg);
        }
        unreachable!()
    }

    // unknown
    fn parse_unknown(s: &str) -> Self {
        Self::Unknown(s.to_owned())
    }

    // usi
    fn parse_usi() -> Result<Self, ParseError> {
        Ok(Self::Usi)
    }

    // debug
    fn parse_debug(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let on = !as_string!(pair).ends_with("off");
        Ok(Self::Debug(on))
    }

    // isready
    fn parse_isready() -> Result<Self, ParseError> {
        Ok(Self::IsReady)
    }

    // setoption
    fn parse_setoption(pair: Pair<Rule>) -> Result<Self, ParseError> {
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
                _ => (),
            }
        }
        Ok(Self::SetOption { name, value })
    }

    // register
    fn parse_register(pair: Pair<Rule>) -> Result<Self, ParseError> {
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
        Ok(Self::Register { name, code })
    }

    // usinewgame
    fn parse_usinewgame() -> Result<Self, ParseError> {
        Ok(Self::UsiNewGame)
    }

    // position
    fn parse_position(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut sfen: Option<String> = None;
        let mut moves: Option<Vec<Move>> = None;
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::startpos => {
                    assert!(sfen.is_none());
                }
                Rule::sfenpos => {
                    sfen = Some(as_string!(sp));
                }
                Rule::moves => {
                    moves = Some(parse_moves(sp)?);
                }
                _ => unreachable!(),
            }
        }
        Ok(Self::Position { sfen, moves })
    }

    // go
    fn parse_go(pair: Pair<Rule>) -> Result<Self, ParseError> {
        // let msg: String = as_string!(pair);
        let mut params = EngineParams::new();

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::searchmoves => {
                    params = params.searchmoves(parse_moves(sp)?);
                }
                Rule::depth => {
                    params = params.depth(parse_digits::<u16>(sp)?);
                }
                Rule::nodes => {
                    params = params.nodes(parse_digits::<u32>(sp)?);
                }
                Rule::mate => { // TODO: either digits or "infinite"
                }
                Rule::byoyomi => {
                    params = params.byoyomi(parse_millisecs(sp)?);
                }
                Rule::btime => {
                    params = params.btime(parse_millisecs(sp)?);
                }
                Rule::wtime => {
                    params = params.wtime(parse_millisecs(sp)?);
                }
                Rule::binc => {
                    params = params.binc(parse_millisecs(sp)?);
                }
                Rule::winc => {
                    params = params.winc(parse_millisecs(sp)?);
                }
                Rule::movestogo => {
                    params = params.movestogo(parse_digits::<u16>(sp)?);
                }

                // implicit assumption is that these are alternatives   TODO: double-check
                Rule::ponder => {
                    params = params.ponder();
                }
                Rule::movetime => params = params.movetime(parse_millisecs(sp)?),
                Rule::infinite => {
                    params = params.infinite();
                }
                _ => unreachable!(),
            }
        }
        Ok(Self::Go(params))
    }

    // stop
    fn parse_stop() -> Result<Self, ParseError> {
        Ok(Self::Stop)
    }

    // ponderhit
    fn parse_ponderhit() -> Result<Self, ParseError> {
        Ok(Self::PonderHit)
    }

    // gameover
    fn parse_gameover(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut status: Option<GameStatus> = None;
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::win => status = Some(GameStatus::Win),
                Rule::lose => status = Some(GameStatus::Lose),
                Rule::draw => status = Some(GameStatus::Draw),
                _ => (),
            }
        }
        if let Some(status) = status {
            Ok(Self::GameOver(status))
        } else {
            Err(ParseError::SyntaxError)
        }
    }

    // quit
    fn parse_quit() -> Result<Self, ParseError> {
        Ok(Self::Quit)
    }
}

impl EngineMessage {
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let pairs: Pairs<Rule> = UsiParser::parse(Rule::start, s)?;

        if let Some(p) = pairs.into_iter().next() {
            let msg: Self = match p.as_rule() {
                Rule::id => Self::parse_id(p)?,
                Rule::usiok => Self::parse_usiok()?,
                Rule::readyok => Self::parse_readyok()?,
                Rule::bestmove => Self::parse_bestmove(p)?,
                Rule::copyprotection => Self::parse_copyprotection(p)?,
                Rule::registration => Self::parse_registration(p)?,
                Rule::option => Self::parse_option(p)?,
                Rule::info => Self::parse_info(p)?,
                _ => Self::parse_unknown(p.as_str()),
            };
            return Ok(msg);
        }
        unreachable!()
    }

    // unknown
    fn parse_unknown(s: &str) -> Self {
        Self::Unknown(s.to_owned())
    }

    // id
    fn parse_id(pair: Pair<Rule>) -> Result<Self, ParseError> {
        if let Some(sp) = pair.into_inner().next() {
            let msg: EngineMessage = match sp.as_rule() {
                Rule::id_name => EngineMessage::Id(IdParams::Name(as_string!(sp))),
                Rule::id_author => EngineMessage::Id(IdParams::Author(as_string!(sp))),
                _ => unreachable!(),
            };
            return Ok(msg);
        }
        Err(ParseError::SyntaxError)
    }

    // usiok
    fn parse_usiok() -> Result<Self, ParseError> {
        Ok(EngineMessage::UsiOk)
    }

    // readyok
    fn parse_readyok() -> Result<Self, ParseError> {
        Ok(EngineMessage::ReadyOk)
    }

    // bestmove
    fn parse_bestmove(pair: Pair<Rule>) -> Result<Self, ParseError> {
        // TODO: grammar change

        let mut bestmove: Option<Move> = None;
        let mut ponder: Option<Move> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::bestmove => {
                    bestmove = Some(Move::from_str(as_str!(sp)).unwrap());
                }
                Rule::ponder_move => {
                    ponder = Some(Move::from_str(as_str!(sp)).unwrap());
                }
                _ => unreachable!(),
            }
        }

        if let Some(bestmove) = bestmove {
            Ok(EngineMessage::BestMove(BestMoveParams::BestMove {
                bestmove,
                ponder,
            }))
        } else {
            Err(ParseError::SyntaxError)
        }
    }

    // copyprotection
    fn parse_copyprotection(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let state = Self::parse_status_check(pair)?;
        Ok(EngineMessage::CopyProtection(state))
    }

    // registration
    fn parse_registration(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let state = Self::parse_status_check(pair)?;
        Ok(EngineMessage::Registration(state))
    }

    fn parse_status_check(pair: Pair<Rule>) -> Result<StatusCheck, ParseError> {
        for sp in pair.into_inner() {
            if let Rule::status_check = sp.as_rule() {
                let s = as_str!(sp);
                match s {
                    "checking" => return Ok(StatusCheck::Checking),
                    "ok" => return Ok(StatusCheck::Ok),
                    "error" => return Ok(StatusCheck::Error),
                    _ => break,
                };
            }
        }
        Err(ParseError::SyntaxError)
    }

    // option
    fn parse_option(pair: Pair<Rule>) -> Result<Self, ParseError> {
        if let Some(sp) = pair.into_inner().next() {
            match sp.as_rule() {
                Rule::check_option => return Self::parse_check_option(sp),
                Rule::spin_option => return Self::parse_spin_option(sp),
                Rule::combo_option => return Self::parse_combo_option(sp),
                Rule::string_option => return Self::parse_string_option(sp),
                Rule::button_option => return Self::parse_button_option(sp),
                Rule::filename_option => return Self::parse_filename_option(sp),
                _ => (),
            }
        }
        Err(ParseError::SyntaxError)
    }

    fn parse_check_option(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut name: Option<String> = None;
        let mut default: Option<bool> = None;
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::option_name => name = Some(as_string!(sp)),
                Rule::check_default => default = Some(as_string!(sp).eq_ignore_ascii_case("true")),
                _ => (),
            }
        }
        if let Some(name) = name {
            Ok(Self::Option(OptionParam::Check { name, default }))
        } else {
            Err(ParseError::SyntaxError)
        }
    }

    fn parse_spin_option(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut name: Option<String> = None;
        let mut default: Option<i32> = None;
        let mut min: Option<i32> = None;
        let mut max: Option<i32> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::option_name => name = Some(as_string!(sp)),
                Rule::spin_default => default = Some(parse_integer::<i32>(sp)?),
                Rule::spin_min => min = Some(parse_integer::<i32>(sp)?),
                Rule::spin_max => max = Some(parse_integer::<i32>(sp)?),
                _ => (),
            }
        }

        if let Some(name) = name {
            Ok(Self::Option(OptionParam::Spin {
                name,
                default,
                min,
                max,
            }))
        } else {
            Err(ParseError::SyntaxError)
        }
    }

    fn parse_combo_option(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut name: Option<String> = None;
        let mut default: Option<String> = None;
        let mut vars: Vec<String> = Vec::new();

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::option_name => name = Some(as_string!(sp)),
                Rule::default => default = Some(as_string!(sp)),
                Rule::var => vars.push(as_string!(sp)),
                _ => (),
            }
        }

        if let Some(name) = name {
            Ok(Self::Option(OptionParam::Combo {
                name,
                default,
                vars,
            }))
        } else {
            Err(ParseError::SyntaxError)
        }
    }

    fn parse_string_option(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut name: Option<String> = None;
        let mut default: Option<String> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::option_name => name = Some(as_string!(sp)),
                Rule::default => {
                    default = {
                        let s = as_string!(sp);
                        convert_empty!(s)
                    }
                }
                _ => (),
            }
        }

        if let Some(name) = name {
            Ok(Self::Option(OptionParam::String { name, default }))
        } else {
            Err(ParseError::SyntaxError)
        }
    }

    fn parse_button_option(pair: Pair<Rule>) -> Result<Self, ParseError> {
        for sp in pair.into_inner() {
            if sp.as_rule() == Rule::option_name {
                let name = as_string!(sp);
                return Ok(Self::Option(OptionParam::Button { name }));
            }
        }
        unreachable!()
    }

    fn parse_filename_option(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut name: Option<String> = None;
        let mut default: Option<String> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::option_name => name = Some(as_string!(sp)),
                Rule::default => {
                    default = {
                        let s = as_string!(sp);
                        convert_empty!(s)
                    }
                }
                _ => (),
            }
        }

        if let Some(name) = name {
            Ok(Self::Option(OptionParam::Filename { name, default }))
        } else {
            Err(ParseError::SyntaxError)
        }
    }

    // info
    fn parse_info(pair: Pair<Rule>) -> Result<Self, ParseError> {
        let mut v: Vec<InfoParam> = Vec::<InfoParam>::new();
        for sp in pair.into_inner() {
            let info: InfoParam = match sp.as_rule() {
                // general
                Rule::info_depth => InfoParam::Depth(parse_digits::<u16>(sp)?),
                Rule::info_seldepth => InfoParam::SelDepth(parse_digits::<u16>(sp)?),
                Rule::info_time => InfoParam::Time(parse_millisecs(sp)?),
                Rule::info_nodes => InfoParam::Nodes(parse_digits::<u64>(sp)?),
                Rule::info_currmovenum => InfoParam::CurrMoveNum(parse_digits::<u16>(sp)?),
                Rule::info_currmove => InfoParam::CurrMove(parse_move(sp)?),
                Rule::info_hashfull => InfoParam::HashFull(parse_digits::<u16>(sp)?),
                Rule::info_nps => InfoParam::Nps(parse_digits::<u64>(sp)?),
                Rule::info_cpuload => InfoParam::CpuLoad(parse_digits::<u16>(sp)?),
                Rule::info_multipv => InfoParam::MultiPv(parse_digits::<u16>(sp)?),
                Rule::info_string => InfoParam::String(as_string!(sp)),
                Rule::info_pv => InfoParam::Pv(parse_moves(sp)?),
                Rule::info_refutation => InfoParam::Refutation(parse_moves(sp)?),
                Rule::info_currline => Self::parse_currline(sp)?,
                Rule::info_score_cp => Self::parse_score_cp(sp)?,
                Rule::info_score_mate => Self::parse_score_mate(sp)?,
                _ => unreachable!(),
            };
            v.push(info);
        }
        Ok(EngineMessage::Info(v))
    }

    // info currline ...
    fn parse_currline(pair: Pair<Rule>) -> Result<InfoParam, ParseError> {
        let mut cpu_nr: Option<u16> = None;
        let mut line: Vec<Move> = Vec::<Move>::new();

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::digits => cpu_nr = Some(parse_digits::<u16>(sp)?),
                Rule::moves => line = parse_moves(sp)?,
                _ => unreachable!(),
            }
        }
        Ok(InfoParam::CurrLine { cpu_nr, line })
    }

    // info score cp ...
    fn parse_score_cp(pair: Pair<Rule>) -> Result<InfoParam, ParseError> {
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
            Ok(InfoParam::ScoreCp(value, bound))
        } else {
            Err(ParseError::SyntaxError)
        }
    }

    // info score mate
    fn parse_score_mate(pair: Pair<Rule>) -> Result<InfoParam, ParseError> {
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
                Rule::sign => {}
                Rule::lowerbound => bound = ScoreBound::Lower,
                Rule::upperbound => bound = ScoreBound::Upper,
                _ => unreachable!(), // really?
            }
        }

        if let Some(value) = v {
            Ok(InfoParam::ScoreCp(value, bound))
        } else {
            Err(ParseError::SyntaxError)
        }
    }
}

// HELPERS

fn parse_move(pair: Pair<Rule>) -> Result<Move, MoveParseError> {
    for sp in pair.into_inner() {
        if let Rule::one_move = sp.as_rule() {
            return as_str!(sp).parse::<Move>();
        }
    }
    unreachable!()
}

fn parse_moves(pair: Pair<Rule>) -> Result<Vec<Move>, MoveParseError> {
    let mut moves = Vec::<Move>::new();

    for sp in pair.into_inner() {
        match sp.as_rule() {
            Rule::one_move => {
                let mv = Move::from_str(as_str!(sp))?;
                moves.push(mv);
            }
            Rule::moves => {
                let mvs: Vec<Move> = parse_moves(sp)?;
                moves.extend(mvs);
            }
            _ => unreachable!(),
        }
    }

    Ok(moves)
}

fn parse_digits<T>(pair: Pair<Rule>) -> Result<T, T::Err>
where
    T: FromStr,
    T::Err: Debug,
{
    for sp in pair.into_inner() {
        if let Rule::digits = sp.as_rule() {
            return as_str!(sp).parse::<T>();
        }
    }
    unreachable!()
}

fn parse_integer<T>(pair: Pair<Rule>) -> Result<T, T::Err>
where
    T: FromStr,
    T::Err: Debug,
{
    for sp in pair.into_inner() {
        if let Rule::integer = sp.as_rule() {
            return as_str!(sp).parse::<T>();
        }
    }
    unreachable!()
}

fn parse_millisecs(pair: Pair<Rule>) -> Result<Duration, ParseError> {
    for sp in pair.into_inner() {
        if let Rule::millisecs = sp.as_rule() {
            let milliseconds: i64 = as_str!(sp).parse::<i64>()?;
            return Ok(Duration::milliseconds(milliseconds));
        }
    }
    unreachable!()
}
