// parser.rs

use core::str::FromStr;
use std::fmt::Debug;
use chrono::Duration;
use haitaka_types::Move;
use pest::Parser; // trait
use pest::error::Error;
use pest::iterators::{Pair, Pairs};
use pest_derive::Parser; // procedural macro

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

/// Expand a GuiMessage variant into it's fully qualified name.
macro_rules! gui {
    ($variant:ident) => {
        UsiMessage::UsiGuiToEngine(GuiMessage::$variant)
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
            Rule::usi => gui!(Usi),
            Rule::debug => GuiMessage::parse_debug(pair),
            Rule::isready => gui!(IsReady),
            Rule::setoption => GuiMessage::parse_setoption(pair),
            Rule::register => GuiMessage::parse_register(pair),
            Rule::usinewgame => gui!(UsiNewGame),
            Rule::stop => gui!(Stop),
            Rule::quit => gui!(Quit),
            Rule::ponderhit => gui!(PonderHit),
            Rule::position => GuiMessage::parse_position(pair),
            Rule::go => GuiMessage::parse_go(pair),
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

impl GuiMessage {
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
                    let mut mvs = Vec::<Move>::new();
                    Self::parse_moves::<false>(sp, &mut mvs);
                    moves = Some(mvs);
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

    fn parse_moves<const DEEP: bool>(pair: Pair<Rule>, moves: &mut Vec<Move>) {
        for sp in pair.into_inner() {
            if let Rule::one_move = sp.as_rule() {
                match Move::from_str(spinstr!(sp)) {
                    Ok(mv) => moves.push(mv),
                    Err(err) => eprintln!("Failed to parse move '{}': {}", spinstr!(sp), err),
                }
            } else if DEEP && sp.as_rule() == Rule::moves {
                Self::parse_moves::<true>(sp, moves);
            }
        }
        debug_assert!(!moves.is_empty());        
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
                Rule::searchmoves => Self::parse_moves::<true>(sp, &mut sc.searchmoves),
                Rule::depth => sc.depth = Self::parse_digits::<u16>(sp),
                Rule::nodes => sc.nodes = Self::parse_digits::<u64>(sp),
                Rule::mate => sc.mate = Self::parse_digits::<u16>(sp),

                Rule::byoyomi => { has_tc = true; byoyomi = Self::parse_millisecs(sp) } 
                Rule::btime => { has_tc = true; black_time = Self::parse_millisecs(sp) }
                Rule::wtime => { has_tc = true; white_time = Self::parse_millisecs(sp) }
                Rule::binc => { has_tc = true; black_increment = Self::parse_millisecs(sp) }
                Rule::winc => { has_tc = true; white_increment = Self::parse_millisecs(sp) }
                Rule::movestogo => { has_tc = true; moves_to_go = Self::parse_digits::<u8>(sp) } 

                // implicit assumption is that these are alternatives
                Rule::ponder => time_control = Some(UsiTimeControl::Ponder),
                Rule::movetime => time_control = Some(UsiTimeControl::MoveTime(Self::parse_millisecs(sp).unwrap())),
                Rule::infinite => time_control = Some(UsiTimeControl::Infinite),
                _ => unreachable!()
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
            eprintln!("WARNING: Ignoring time control related subcommands in `{}`", msg);
            eprintln!("Commands `go ponder`, `go infinite` and `go movetime <ms>` should be sent as separate messages.");
        }

        UsiMessage::UsiGuiToEngine(GuiMessage::Go { time_control, search_control })
    }

    fn parse_digits<T: FromStr + Debug>(pair: Pair<Rule>) -> Option<T> 
    where <T as FromStr>::Err: Debug
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

}


