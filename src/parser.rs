// parser.rs

use core::str::FromStr;
// use chrono::Duration;
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
    let pairs = UsiParser::parse(Rule::start, s).unwrap();
    println!("{:#?}", pairs);
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

/// Extract the string value of a PEST Span as String.
/// This will also trim leading and trailing whitespace from the string.
macro_rules! spanstr {
    ($sp:ident) => {
        $sp.as_span().as_str().trim().to_string()
    };
}

/// Extract the string value of a PEST Span as str.
macro_rules! spinstr {
    ($sp:ident) => {
        $sp.as_span().as_str()
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
                    moves = Some(Self::parse_moves(sp));
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

    fn parse_moves(pair: Pair<Rule>) -> Vec<Move> {
        let mut moves = Vec::<Move>::new();
        for sp in pair.into_inner() {
            match sp.as_rule() {
                Rule::one_move => {
                    let mv = Move::from_str(spinstr!(sp)).unwrap();
                    moves.push(mv);
                }
                _ => {}
            }
        }

        moves
    }
}

/*

>> position startpos
[D
    Pair {
        rule: position,
        span: Span {
            str: "position startpos",
            start: 0,
            end: 17,
        },
        inner: [
            Pair {
                rule: startpos,
                span: Span {
                    str: "startpos",
                    start: 9,
                    end: 17,
                },
                inner: [],
            },
        ],
    },
    Pair {
        rule: EOI,
        span: Span {
            str: "",
            start: 18,
            end: 18,
        },
        inner: [],
    },
]

=====



(base) MacBook-Pro:haitaka_usi hansgeuns-meyer$ cargo run
   Compiling haitaka_usi v0.1.0 (/Users/hansgeuns-meyer/Projects/Shogi/haitaka_usi)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.36s
     Running `target/debug/haitaka_usi`
>>> position startpos moves 2g2f 8c8d
[D
    Pair {
        rule: position,
        span: Span {
            str: "position startpos moves 2g2f 8c8d",
            start: 0,
            end: 33,
        },
        inner: [
            Pair {
                rule: startpos,
                span: Span {
                    str: "startpos",
                    start: 9,
                    end: 17,
                },
                inner: [],
            },
            Pair {
                rule: moves,
                span: Span {
                    str: "2g2f 8c8d",
                    start: 24,
                    end: 33,
                },
                inner: [
                    Pair {
                        rule: one_move,
                        span: Span {
                            str: "2g2f",
                            start: 24,
                            end: 28,
                        },
                        inner: [
                            Pair {
                                rule: board_move,
                                span: Span {
                                    str: "2g2f",
                                    start: 24,
                                    end: 28,
                                },
                                inner: [
                                    Pair {
                                        rule: square,
                                        span: Span {
                                            str: "2g",
                                            start: 24,
                                            end: 26,
                                        },
                                        inner: [
                                            Pair {
                                                rule: file,
                                                span: Span {
                                                    str: "2",
                                                    start: 24,
                                                    end: 25,
                                                },
                                                inner: [],
                                            },
                                            Pair {
                                                rule: rank,
                                                span: Span {
                                                    str: "g",
                                                    start: 25,
                                                    end: 26,
                                                },
                                                inner: [],
                                            },
                                        ],
                                    },
                                    Pair {
                                        rule: square,
                                        span: Span {
                                            str: "2f",
                                            start: 26,
                                            end: 28,
                                        },
                                        inner: [
                                            Pair {
                                                rule: file,
                                                span: Span {
                                                    str: "2",
                                                    start: 26,
                                                    end: 27,
                                                },
                                                inner: [],
                                            },
                                            Pair {
                                                rule: rank,
                                                span: Span {
                                                    str: "f",
                                                    start: 27,
                                                    end: 28,
                                                },
                                                inner: [],
                                            },
                                        ],
                                    },
                                ],
                            },
                        ],
                    },
                    Pair {
                        rule: one_move,
                        span: Span {
                            str: "8c8d",
                            start: 29,
                            end: 33,
                        },
                        inner: [
                            Pair {
                                rule: board_move,
                                span: Span {
                                    str: "8c8d",
                                    start: 29,
                                    end: 33,
                                },
                                inner: [
                                    Pair {
                                        rule: square,
                                        span: Span {
                                            str: "8c",
                                            start: 29,
                                            end: 31,
                                        },
                                        inner: [
                                            Pair {
                                                rule: file,
                                                span: Span {
                                                    str: "8",
                                                    start: 29,
                                                    end: 30,
                                                },
                                                inner: [],
                                            },
                                            Pair {
                                                rule: rank,
                                                span: Span {
                                                    str: "c",
                                                    start: 30,
                                                    end: 31,
                                                },
                                                inner: [],
                                            },
                                        ],
                                    },
                                    Pair {
                                        rule: square,
                                        span: Span {
                                            str: "8d",
                                            start: 31,
                                            end: 33,
                                        },
                                        inner: [
                                            Pair {
                                                rule: file,
                                                span: Span {
                                                    str: "8",
                                                    start: 31,
                                                    end: 32,
                                                },
                                                inner: [],
                                            },
                                            Pair {
                                                rule: rank,
                                                span: Span {
                                                    str: "d",
                                                    start: 32,
                                                    end: 33,
                                                },
                                                inner: [],
                                            },
                                        ],
                                    },
                                ],
                            },
                        ],
                    },
                ],
            },
        ],
    },
    Pair {
        rule: EOI,
        span: Span {
            str: "",
            start: 34,
            end: 34,
        },
        inner: [],
    },
]
=================

position sfen ln1g5/1r2S1k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b w BGSLPnp 62
[D
    Pair {
        rule: position,
        span: Span {
            str: "position sfen ln1g5/1r2S1k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b w B",
            start: 0,
            end: 79,
        },
        inner: [
            Pair {
                rule: sfenpos,
                span: Span {
                    str: "sfen ln1g5/1r2S1k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b w B",
                    start: 9,
                    end: 79,
                },
                inner: [
                    Pair {
                        rule: sfen_board,
                        span: Span {
                            str: "ln1g5/1r2S1k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b",
                            start: 14,
                            end: 75,
                        },
                        inner: [
                            Pair {
                                rule: sfen_rank,
                                span: Span {
                                    str: "ln1g5",
                                    start: 14,
                                    end: 19,
                                },
                                inner: [
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "l",
                                            start: 14,
                                            end: 15,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "n",
                                            start: 15,
                                            end: 16,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "1",
                                            start: 16,
                                            end: 17,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "g",
                                            start: 17,
                                            end: 18,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "5",
                                            start: 18,
                                            end: 19,
                                        },
                                        inner: [],
                                    },
                                ],
                            },
                            Pair {
                                rule: sfen_rank,
                                span: Span {
                                    str: "1r2S1k2",
                                    start: 20,
                                    end: 27,
                                },
                                inner: [
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "1",
                                            start: 20,
                                            end: 21,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "r",
                                            start: 21,
                                            end: 22,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "2",
                                            start: 22,
                                            end: 23,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "S",
                                            start: 23,
                                            end: 24,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "1",
                                            start: 24,
                                            end: 25,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "k",
                                            start: 25,
                                            end: 26,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "2",
                                            start: 26,
                                            end: 27,
                                        },
                                        inner: [],
                                    },
                                ],
                            },
                            Pair {
                                rule: sfen_rank,
                                span: Span {
                                    str: "p2pppn2",
                                    start: 28,
                                    end: 35,
                                },
                                inner: [
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "p",
                                            start: 28,
                                            end: 29,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "2",
                                            start: 29,
                                            end: 30,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "p",
                                            start: 30,
                                            end: 31,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "p",
                                            start: 31,
                                            end: 32,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "p",
                                            start: 32,
                                            end: 33,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "n",
                                            start: 33,
                                            end: 34,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "2",
                                            start: 34,
                                            end: 35,
                                        },
                                        inner: [],
                                    },
                                ],
                            },
                            Pair {
                                rule: sfen_rank,
                                span: Span {
                                    str: "2ps2p2",
                                    start: 36,
                                    end: 42,
                                },
                                inner: [
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "2",
                                            start: 36,
                                            end: 37,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "p",
                                            start: 37,
                                            end: 38,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "s",
                                            start: 38,
                                            end: 39,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "2",
                                            start: 39,
                                            end: 40,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "p",
                                            start: 40,
                                            end: 41,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "2",
                                            start: 41,
                                            end: 42,
                                        },
                                        inner: [],
                                    },
                                ],
                            },
                            Pair {
                                rule: sfen_rank,
                                span: Span {
                                    str: "1p7",
                                    start: 43,
                                    end: 46,
                                },
                                inner: [
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "1",
                                            start: 43,
                                            end: 44,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "p",
                                            start: 44,
                                            end: 45,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "7",
                                            start: 45,
                                            end: 46,
                                        },
                                        inner: [],
                                    },
                                ],
                            },
                            Pair {
                                rule: sfen_rank,
                                span: Span {
                                    str: "2P6",
                                    start: 47,
                                    end: 50,
                                },
                                inner: [
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "2",
                                            start: 47,
                                            end: 48,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "P",
                                            start: 48,
                                            end: 49,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "6",
                                            start: 49,
                                            end: 50,
                                        },
                                        inner: [],
                                    },
                                ],
                            },
                            Pair {
                                rule: sfen_rank,
                                span: Span {
                                    str: "PPSPPPPLP",
                                    start: 51,
                                    end: 60,
                                },
                                inner: [
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "P",
                                            start: 51,
                                            end: 52,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "P",
                                            start: 52,
                                            end: 53,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "S",
                                            start: 53,
                                            end: 54,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "P",
                                            start: 54,
                                            end: 55,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "P",
                                            start: 55,
                                            end: 56,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "P",
                                            start: 56,
                                            end: 57,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "P",
                                            start: 57,
                                            end: 58,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "L",
                                            start: 58,
                                            end: 59,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "P",
                                            start: 59,
                                            end: 60,
                                        },
                                        inner: [],
                                    },
                                ],
                            },
                            Pair {
                                rule: sfen_rank,
                                span: Span {
                                    str: "2G2K1pr",
                                    start: 61,
                                    end: 68,
                                },
                                inner: [
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "2",
                                            start: 61,
                                            end: 62,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "G",
                                            start: 62,
                                            end: 63,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "2",
                                            start: 63,
                                            end: 64,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "K",
                                            start: 64,
                                            end: 65,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "1",
                                            start: 65,
                                            end: 66,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "p",
                                            start: 66,
                                            end: 67,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "r",
                                            start: 67,
                                            end: 68,
                                        },
                                        inner: [],
                                    },
                                ],
                            },
                            Pair {
                                rule: sfen_rank,
                                span: Span {
                                    str: "LN4G1b",
                                    start: 69,
                                    end: 75,
                                },
                                inner: [
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "L",
                                            start: 69,
                                            end: 70,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "N",
                                            start: 70,
                                            end: 71,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "4",
                                            start: 71,
                                            end: 72,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: black_piece,
                                        span: Span {
                                            str: "G",
                                            start: 72,
                                            end: 73,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: file,
                                        span: Span {
                                            str: "1",
                                            start: 73,
                                            end: 74,
                                        },
                                        inner: [],
                                    },
                                    Pair {
                                        rule: white_piece,
                                        span: Span {
                                            str: "b",
                                            start: 74,
                                            end: 75,
                                        },
                                        inner: [],
                                    },
                                ],
                            },
                        ],
                    },
                    Pair {
                        rule: sfen_color,
                        span: Span {
                            str: "w",
                            start: 76,
                            end: 77,
                        },
                        inner: [],
                    },
                    Pair {
                        rule: sfen_hands,
                        span: Span {
                            str: "B",
                            start: 78,
                            end: 79,
                        },
                        inner: [
                            Pair {
                                rule: sfen_black_hand,
                                span: Span {
                                    str: "B",
                                    start: 78,
                                    end: 79,
                                },
                                inner: [
                                    Pair {
                                        rule: black_hand_piece,
                                        span: Span {
                                            str: "B",
                                            start: 78,
                                            end: 79,
                                        },
                                        inner: [],
                                    },
                                ],
                            },
                        ],
                    },
                ],
            },
        ],
    },
    Pair {
        rule: EOI,
        span: Span {
            str: "",
            start: 89,
            end: 89,
        },
        inner: [],
    },
]

Unknown(
    "position sfen ln1g5/1r2S1k2/p2pppn2/2ps2p2/1p7/2P6/PPSPPPPLP/2G2K1pr/LN4G1b w B",
    None,
)
>>>



>> register    later
[D
    Pair {
        rule: register,
        span: Span {
            str: "register    later",
            start: 0,
            end: 17,
        },
        inner: [
            Pair {
                rule: register_later,
                span: Span {
                    str: "register    later",
                    start: 0,
                    end: 17,
                },
                inner: [],
            },
        ],
    },
    Pair {
        rule: EOI,
        span: Span {
            str: "",
            start: 18,
            end: 18,
        },
        inner: [],
    },
]


> register name A B code 123 456 yoho
[D
    Pair {
        rule: register,
        span: Span {
            str: "register name A B code 123 456 yoho",
            start: 0,
            end: 35,
        },
        inner: [
            Pair {
                rule: register_with_name_and_code,
                span: Span {
                    str: "register name A B code 123 456 yoho",
                    start: 0,
                    end: 35,
                },
                inner: [
                    Pair {
                        rule: register_name,
                        span: Span {
                            str: "A B",
                            start: 14,
                            end: 17,
                        },
                        inner: [
                            Pair {
                                rule: token,
                                span: Span {
                                    str: "A",
                                    start: 14,
                                    end: 15,
                                },
                                inner: [],
                            },
                            Pair {
                                rule: token,
                                span: Span {
                                    str: "B",
                                    start: 16,
                                    end: 17,
                                },
                                inner: [],
                            },
                        ],
                    },
                    Pair {
                        rule: register_code,
                        span: Span {
                            str: "123 456 yoho",
                            start: 23,
                            end: 35,
                        },
                        inner: [
                            Pair {
                                rule: tokens,
                                span: Span {
                                    str: "123 456 yoho",
                                    start: 23,
                                    end: 35,
                                },
                                inner: [],
                            },
                        ],
                    },
                ],
            },
        ],
    },
    Pair {
        rule: EOI,
        span: Span {
            str: "",
            start: 36,
            end: 36,
        },
        inner: [],
    },
]





Pair {
    rule: setoption,
    span: Span {
        str: "setoption name A B C value X",
        start: 0,
        end: 28,
    },
    inner: [
        Pair {
            rule: setoption_name,
            span: Span {
                str: "A B C",
                start: 15,
                end: 20,
            },
            inner: [
                Pair {
                    rule: token,
                    span: Span {
                        str: "A",
                        start: 15,
                        end: 16,
                    },
                    inner: [],
                },
                Pair {
                    rule: token,
                    span: Span {
                        str: "B",
                        start: 17,
                        end: 18,
                    },
                    inner: [],
                },
                Pair {
                    rule: token,
                    span: Span {
                        str: "C",
                        start: 19,
                        end: 20,
                    },
                    inner: [],
                },
            ],
        },
        Pair {
            rule: setoption_value,
            span: Span {
                str: "X",
                start: 27,
                end: 28,
            },
            inner: [
                Pair {
                    rule: tokens,
                    span: Span {
                        str: "X",
                        start: 27,
                        end: 28,
                    },
                    inner: [],
                },
            ],
        },
    ],
},
Pair {
    rule: EOI,
    span: Span {
        str: "",
        start: 29,
        end: 29,
    },
    inner: [],
},
*/
