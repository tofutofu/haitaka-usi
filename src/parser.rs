// parser.rs

use chrono::Duration;
use core::str::FromStr;
use haitaka_types::Move;
use pest::Parser; // trait
use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser;
use std::fmt::Debug; // procedural macro

use crate::usi::*;

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

pub fn parse(s: &str) -> UsiMessageList {
    let mut messages = UsiMessageList::new();
    parse_usi(s, Rule::start, Some(&mut messages)).unwrap();

    messages
}

pub fn try_parse(s: &str) -> Result<UsiMessageList, Error<Rule>> {
    let mut messages = UsiMessageList::new();
    parse_usi(s, Rule::start, Some(&mut messages))?;

    Ok(messages)
}

pub fn parse_one(s: &str) -> UsiMessage {
    let res = parse_usi(s, Rule::start, None);

    if let Err(err) = res {
        let msg = UsiMessage::Unknown(s.trim_end().to_owned(), Some(err));
        return msg;
    }

    if let Some(msg) = res.unwrap() {
        return msg;
    }

    return UsiMessage::Unknown(String::new(), None);
}

/// Expand a GuiMessage variant into it's full name.
macro_rules! gui {
    ($variant:ident) => {
        UsiMessage::UsiGuiToEngine(GuiMessage::$variant)
    };
}

/// Expand an EngineMessage variant into it's full name
macro_rules! engine {
    ($variant:ident) => {
        UsiMessage::UsiEngineToGui(EngineMessage::$variant)
    };
}

/// Extract the string value of a PEST Span as `String`.
/// Also trims leading and trailing whitespace.
macro_rules! spanstr {
    ($sp:ident) => {
        $sp.as_span().as_str().trim().to_string()
    };
}

/// Extract the string value of a PEST Span as `str`.
/// Also trims leading and trailing whitespace.
macro_rules! spinstr {
    ($sp:ident) => {
        $sp.as_span().as_str().trim()
    };
}

fn parse_usi(
    s: &str,
    rule: Rule,
    mut messages: Option<&mut UsiMessageList>,
) -> Result<Option<UsiMessage>, Error<Rule>> {
    let pairs: Pairs<Rule> = UsiParser::parse(rule, s)?;

    for pair in pairs {
        let msg = match pair.as_rule() {
            // gui-to-engine
            Rule::usi => gui!(Usi),
            Rule::debug => UsiMessage::parse_debug(pair),
            Rule::isready => gui!(IsReady),
            Rule::setoption => UsiMessage::parse_setoption(pair),
            Rule::register => UsiMessage::parse_register(pair),
            Rule::usinewgame => gui!(UsiNewGame),
            Rule::stop => gui!(Stop),
            Rule::quit => gui!(Quit),
            Rule::ponderhit => gui!(PonderHit),
            Rule::position => UsiMessage::parse_position(pair),
            Rule::go => UsiMessage::parse_go(pair),
            // engine-to-gui
            Rule::id => UsiMessage::parse_id(pair),
            Rule::usiok => engine!(UsiOk),
            Rule::readyok => engine!(ReadyOk),
            Rule::bestmove => UsiMessage::parse_bestmove(pair),
            Rule::copyprotection => UsiMessage::parse_copyprotection(pair),
            Rule::registration => UsiMessage::parse_registration(pair),
            Rule::option => UsiMessage::parse_option(pair),
            Rule::info => UsiMessage::parse_info(pair),
            _ => UsiMessage::Unknown(spanstr!(pair), None),
        };

        if let Some(msgs) = &mut messages {
            (*msgs).push(msg);
        } else {
            return Ok(Some(msg));
        }
    }

    Ok(None)
}

impl UsiMessage {
    //
    // gui-to-engine
    //
    pub fn parse_debug(pair: Pair<Rule>) -> UsiMessage {
        let on = !pair.as_span().as_str().trim_end().ends_with("off");
        UsiMessage::UsiGuiToEngine(GuiMessage::Debug(on))
    }

    pub fn parse_setoption(pair: Pair<Rule>) -> UsiMessage {
        let mut name: String = String::default();
        let mut value: String = String::default();
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::setoption_name => {
                    name = spanstr!(sp);
                }
                Rule::setoption_value => {
                    value = spanstr!(sp);
                }
                _ => {}
            }
        }
        let value = if value != String::default() {
            Some(value)
        } else {
            None
        };
        UsiMessage::UsiGuiToEngine(GuiMessage::SetOption { name, value })
    }

    pub fn parse_register(pair: Pair<Rule>) -> UsiMessage {
        let mut later: bool = false;
        let mut name: Option<String> = None;
        let mut code: Option<String> = None;
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::register_later => {
                    later = true;
                }
                Rule::register_with_name_and_code => {
                    for spi in sp.into_inner() {
                        match spi.as_rule() {
                            Rule::register_name => {
                                name = Some(spanstr!(spi));
                            }
                            Rule::register_code => {
                                code = Some(spanstr!(spi));
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                _ => unreachable!(),
            }
        }
        UsiMessage::UsiGuiToEngine(GuiMessage::Register { later, name, code })
    }

    pub fn parse_position(pair: Pair<Rule>) -> UsiMessage {
        let mut startpos: bool = false;
        let mut sfen: Option<String> = None;
        let mut moves: Option<Vec<Move>> = None;
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::startpos => {
                    assert!(sfen.is_none());
                    startpos = true;
                }
                Rule::sfenpos => {
                    assert!(!startpos);
                    sfen = Some(spanstr!(sp));
                }
                Rule::moves => {
                    moves = Some(Self::parse_moves::<false>(sp));
                }
                _ => unreachable!(),
            }
        }
        UsiMessage::UsiGuiToEngine(GuiMessage::Position {
            startpos,
            sfen,
            moves,
        })
    }

    fn parse_moves_inplace<const DEEP: bool>(pair: Pair<Rule>, moves: &mut Vec<Move>) {
        for sp in pair.into_inner() {
            if let Rule::one_move = sp.as_rule() {
                // REVIEW: The grammar should really already guard against errors,
                // so should I simply use from_str(...).ok() ?
                match Move::from_str(spinstr!(sp)) {
                    Ok(mv) => moves.push(mv),
                    Err(err) => eprintln!("Failed to parse move '{}': {}", spinstr!(sp), err),
                }
            } else if DEEP && sp.as_rule() == Rule::moves {
                Self::parse_moves_inplace::<true>(sp, moves);
            }
        }
        debug_assert!(!moves.is_empty());
    }

    fn parse_moves<const DEEP: bool>(pair: Pair<Rule>) -> Vec<Move> {
        let mut moves = Vec::<Move>::new();

        for sp in pair.into_inner() {
            if let Rule::one_move = sp.as_rule() {
                match Move::from_str(spinstr!(sp)) {
                    Ok(mv) => moves.push(mv),
                    Err(err) => eprintln!("Failed to parse move '{}': {}", spinstr!(sp), err),
                }
            } else if DEEP && sp.as_rule() == Rule::moves {
                // Recursive call with the same optional vector
                Self::parse_moves_inplace::<DEEP>(sp, &mut moves);
            }
        }

        moves
    }

    fn parse_one_move(pair: Pair<Rule>) -> Move {
        for sp in pair.into_inner() {
            if let Rule::one_move = sp.as_rule() {
                match Move::from_str(spinstr!(sp)) {
                    Ok(mv) => return mv,
                    Err(err) => eprintln!("Failed to parse move '{}': {}", spinstr!(sp), err),
                }
            }
        }
        unreachable!();
    }

    pub fn parse_go(pair: Pair<Rule>) -> UsiMessage {
        let msg: String = spanstr!(pair);

        let mut time_control: Option<UsiTimeControl> = None;
        let mut search_control: Option<UsiSearchControl> = None;

        let mut sc = UsiSearchControl::default();
        let mut has_tc = false;
        let mut byoyomi: Option<Duration> = None;
        let mut black_time: Option<Duration> = None;
        let mut white_time: Option<Duration> = None;
        let mut black_increment: Option<Duration> = None;
        let mut white_increment: Option<Duration> = None;
        let mut moves_to_go: Option<u8> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::searchmoves => Self::parse_moves_inplace::<true>(sp, &mut sc.searchmoves),
                Rule::depth => sc.depth = Self::parse_digits::<u16>(sp),
                Rule::nodes => sc.nodes = Self::parse_digits::<u64>(sp),
                Rule::mate => sc.mate = Self::parse_digits::<u16>(sp),

                Rule::byoyomi => {
                    has_tc = true;
                    byoyomi = Self::parse_millisecs(sp)
                }
                Rule::btime => {
                    has_tc = true;
                    black_time = Self::parse_millisecs(sp)
                }
                Rule::wtime => {
                    has_tc = true;
                    white_time = Self::parse_millisecs(sp)
                }
                Rule::binc => {
                    has_tc = true;
                    black_increment = Self::parse_millisecs(sp)
                }
                Rule::winc => {
                    has_tc = true;
                    white_increment = Self::parse_millisecs(sp)
                }
                Rule::movestogo => {
                    has_tc = true;
                    moves_to_go = Self::parse_digits::<u8>(sp)
                }

                // implicit assumption is that these are alternatives   TODO: double-check
                Rule::ponder => time_control = Some(UsiTimeControl::Ponder),
                Rule::movetime => {
                    time_control =
                        Some(UsiTimeControl::MoveTime(Self::parse_millisecs(sp).unwrap()))
                }
                Rule::infinite => time_control = Some(UsiTimeControl::Infinite),
                _ => unreachable!(),
            }
        }

        if sc.is_active() {
            search_control = Some(sc);
        }

        if time_control.is_none() && has_tc {
            time_control = Some(UsiTimeControl::TimeLeft {
                white_time,
                black_time,
                white_increment,
                black_increment,
                moves_to_go,
                byoyomi,
            });
        } else if has_tc {
            // TODO: Check this against Stockfish/Apery/YaneuraOu

            // The currently implemented TimeControl enum is not able to handle this.
            // If this really is an error, shouldn't the grammar this forbid it on the syntax level?
            // e.g. `go ponder mate 15`
            eprintln!(
                "WARNING: Ignoring time control related subcommands in `{}`",
                msg
            );
            eprintln!(
                "Commands `go ponder`, `go infinite` and `go movetime <ms>` should be sent as separate messages."
            );
        }

        UsiMessage::UsiGuiToEngine(GuiMessage::Go {
            time_control,
            search_control,
        })
    }

    fn parse_digits<T: FromStr + Debug>(pair: Pair<Rule>) -> Option<T>
    where
        <T as FromStr>::Err: Debug,
    {
        for sp in pair.into_inner() {
            if let Rule::digits = sp.as_rule() {
                let digits_str = spinstr!(sp); // Extract the str representation of the digits
                match digits_str.parse::<T>() {
                    Ok(value) => return Some(value),
                    Err(err) => eprintln!("Failed to parse digits '{}': {:?}", digits_str, err),
                }
            }
        }
        None // unreachable!()
    }

    /*
    fn parse_digits<T: FromStr + Debug>(pair: Pair<Rule>) -> Result<T, String> {
        for sp in pair.into_inner() {
            if let Rule::digits = sp.as_rule() {
                let digits_str = spinstr!(sp); // Extract the string representation of the digits
                return digits_str.parse::<T>().map_err(|err| {
                    format!("Failed to parse digits '{}': {:?}", digits_str, err)
                });
            }
        }
        Err("Expected Rule::digits but found none".to_string())
    }
    */

    fn parse_millisecs(pair: Pair<Rule>) -> Option<Duration> {
        for sp in pair.into_inner() {
            if let Rule::millisecs = sp.as_rule() {
                let s: &str = spinstr!(sp);
                match s.parse::<i64>() {
                    Ok(millis) => return Some(Duration::milliseconds(millis)),
                    Err(err) => eprintln!("Failed to parse milliseconds '{}': {:?}", s, err),
                }
            }
        }
        None // unreachable!()
    }

    // engine-to-gui

    pub fn parse_id(pair: Pair<Rule>) -> UsiMessage {
        let mut name: Option<String> = None;
        let mut author: Option<String> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::id_name => {
                    name = Some(spanstr!(sp));
                }
                Rule::id_author => {
                    author = Some(spanstr!(sp));
                }
                _ => unreachable!(),
            }
        }

        UsiMessage::UsiEngineToGui(EngineMessage::Id { name, author })
    }

    pub fn parse_bestmove(pair: Pair<Rule>) -> UsiMessage {
        let mut best_move: Move = Move::from_str("1a1a").unwrap(); // an invalid default move
        let mut ponder: Option<Move> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::bestmove => {
                    best_move = Move::from_str(spinstr!(sp)).unwrap();
                }
                Rule::ponder_move => {
                    ponder = Some(Move::from_str(spinstr!(sp)).unwrap());
                }
                _ => unreachable!(),
            }
        }

        UsiMessage::UsiEngineToGui(EngineMessage::BestMove { best_move, ponder })
    }

    pub fn parse_copyprotection(pair: Pair<Rule>) -> UsiMessage {
        let state = Self::parse_status_check(pair);
        UsiMessage::UsiEngineToGui(EngineMessage::CopyProtection(state))
    }

    pub fn parse_registration(pair: Pair<Rule>) -> UsiMessage {
        let state = Self::parse_status_check(pair);
        UsiMessage::UsiEngineToGui(EngineMessage::Registration(state))
    }

    fn parse_status_check(pair: Pair<Rule>) -> StatusCheck {
        for sp in pair.into_inner() {
            if let Rule::status_check = sp.as_rule() {
                match spinstr!(sp) {
                    "checking" => return StatusCheck::Checking,
                    "ok" => return StatusCheck::Ok,
                    "error" => return StatusCheck::Error,
                    _ => unreachable!(),
                };
            }
        }
        unreachable!()
    }

    pub fn parse_option(_pair: Pair<Rule>) -> UsiMessage {
        UsiMessage::Unknown("option".to_string(), None)
    }

    pub fn parse_info(pair: Pair<Rule>) -> UsiMessage {
        let mut v: Vec<UsiInfo> = Vec::<UsiInfo>::new();
        for sp in pair.into_inner() {
            let info: UsiInfo = match sp.as_rule() {
                // general
                Rule::info_depth => UsiInfo::Depth(Self::parse_digits::<u16>(sp).unwrap()),
                Rule::info_seldepth => UsiInfo::SelDepth(Self::parse_digits::<u16>(sp).unwrap()),
                Rule::info_time => UsiInfo::Time(Self::parse_millisecs(sp).unwrap()),
                Rule::info_nodes => UsiInfo::Nodes(Self::parse_digits::<u64>(sp).unwrap()),
                Rule::info_currmovenum => {
                    UsiInfo::CurrMoveNum(Self::parse_digits::<u16>(sp).unwrap())
                }
                Rule::info_currmove => UsiInfo::CurrMove(Self::parse_one_move(sp)),
                Rule::info_hashfull => UsiInfo::HashFull(Self::parse_digits::<u16>(sp).unwrap()),
                Rule::info_nps => UsiInfo::Nps(Self::parse_digits::<u64>(sp).unwrap()),
                Rule::info_cpuload => UsiInfo::CpuLoad(Self::parse_digits::<u16>(sp).unwrap()),
                Rule::info_multipv => UsiInfo::MultiPv(Self::parse_digits::<u16>(sp).unwrap()),
                Rule::info_string => UsiInfo::String(spanstr!(sp)),
                // lines
                Rule::info_pv => UsiInfo::Pv(Self::parse_moves::<true>(sp)),
                Rule::info_refutation => UsiInfo::Refutation(Self::parse_moves::<true>(sp)),
                Rule::info_currline => Self::parse_currline(sp),
                // score
                Rule::info_score => Self::parse_score(sp),
                _ => unreachable!(),
            };
            v.push(info);
        }
        UsiMessage::UsiEngineToGui(EngineMessage::Info(v))
    }

    fn parse_currline(pair: Pair<Rule>) -> UsiInfo {
        let mut cpu_nr: Option<u16> = None;
        let mut line: Vec<Move> = Vec::<Move>::new();

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::digits => {
                    cpu_nr = Some(Self::parse_digits::<u16>(sp).unwrap());
                }
                Rule::moves => {
                    Self::parse_moves_inplace::<false>(sp, &mut line);
                }
                _ => unreachable!(),
            }
        }
        UsiInfo::CurrLine { cpu_nr, line }
    }

    fn parse_score(pair: Pair<Rule>) -> UsiInfo {
        let mut cp: Option<i32> = None;
        let mut mate: Option<i16> = None;
        let mut lowerbound: Option<bool> = None;
        let mut upperbound: Option<bool> = None;

        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::score_cp => {
                    cp = Some(spinstr!(sp).parse::<i32>().unwrap());
                }
                Rule::score_mate => {
                    mate = Some(spinstr!(sp).parse::<i16>().unwrap());
                }
                Rule::score_lowerbound => {
                    lowerbound = Some(true);
                }
                Rule::score_upperbound => {
                    upperbound = Some(true);
                }
                _ => unreachable!(),
            }
        }
        UsiInfo::Score {
            cp,
            mate,
            lowerbound,
            upperbound,
        }
    }
}
