#[cfg(test)]
mod tests {
    use crate::*;
    use haitaka_types::{Move, Square};
    use std::time::Duration;

    fn s(s: &str) -> String {
        s.to_owned()
    }

    //
    // GUI
    //

    #[test]
    fn test_gui_usi() {
        let msg = GuiMessage::parse("usi\n").unwrap();
        assert_eq!(msg, GuiMessage::Usi);
    }

    #[test]
    fn test_gui_usinewgame() {
        let msg = GuiMessage::parse("usinewgame\n").unwrap();
        assert_eq!(msg, GuiMessage::UsiNewGame);
    }

    #[test]
    fn test_gui_usi_unknown() {
        let msg = GuiMessage::parse("usi yoho\n").unwrap();
        assert_eq!(msg, GuiMessage::Unknown(s("usi yoho\n")));
    }

    #[test]
    fn test_gui_usi_prefix() {
        let msg = GuiMessage::parse("yoho usi\n").unwrap();
        assert_eq!(msg, GuiMessage::Unknown(s("yoho ")));
    }

    #[test]
    fn test_gui_first_valid() {
        let msg = GuiMessage::parse_first_valid("yoho\nhey usi \n").unwrap();
        assert_eq!(msg, GuiMessage::Usi);
    }

    #[test]
    fn test_gui_usi_missing_newline() {
        GuiMessage::parse("usi").expect_err("Protocol messages require a newline at the end");
    }

    #[test]
    fn test_gui_usi_cr() {
        let input = "usi\r";
        let msg = GuiMessage::parse(input).unwrap();
        assert_eq!(msg, GuiMessage::Usi);
    }

    #[test]
    fn test_gui_usi_crnl() {
        let input = "usi\r\n";
        let msg = GuiMessage::parse(input).unwrap();
        assert_eq!(msg, GuiMessage::Usi);
    }

    #[test]
    fn test_gui_first_valid_missing_newline() {
        let result = std::panic::catch_unwind(|| {
            GuiMessage::parse_first_valid("yoho\nhey usi");
        });
        assert!(
            result.is_err(),
            "Expected a panic attack (missing newline), but none occurred"
        );
    }

    //
    // roundtrip tests
    //

    #[test]
    fn test_gui_roundtrip_usi() {
        let msg = GuiMessage::Usi;
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_usinewgame() {
        let msg = GuiMessage::UsiNewGame;
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_debug_on() {
        let msg = GuiMessage::Debug(true);
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_debug_off() {
        let msg = GuiMessage::Debug(false);
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_isready() {
        let msg = GuiMessage::IsReady;
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_setoption_with_value() {
        let msg = GuiMessage::SetOption {
            name: s("USI_Hash"),
            value: Some(s("128")),
        };
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_setoption_without_value() {
        let msg = GuiMessage::SetOption {
            name: s("USI_Ponder"),
            value: None,
        };
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_register_later() {
        let msg = GuiMessage::Register {
            name: None,
            code: None,
        };
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_register_with_name_and_code() {
        let msg = GuiMessage::Register {
            name: Some(s("Fee Fie Foo")),
            code: Some(s("123 x 456")),
        };
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_stop() {
        let msg = GuiMessage::Stop;
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_quit() {
        let msg = GuiMessage::Quit;
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_gameover() {
        let msg = GuiMessage::GameOver(GameStatus::Win);
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);

        let msg = GuiMessage::GameOver(GameStatus::Lose);
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);

        let msg = GuiMessage::GameOver(GameStatus::Draw);
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), s);
    }

    #[test]
    fn test_gui_roundtrip_position_startpos() {
        let sfen: Option<String> = None;
        let moves: Option<Vec<Move>> = None;
        let msg = GuiMessage::Position { sfen, moves };
        let s = format!("{msg}\n");
        assert_eq!(s, "position startpos\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_position_sfen() {
        let input = "position sfen 8l/1l+R2P3/p2pBG1pp/kps1p4/Nn1P2G2/P1P1P2PP/1PS6/1KSG3+r1/LN2+p3L w Sbgn3p 124\n";
        let msg = GuiMessage::parse(&input).unwrap();
        let output = format!("{msg}\n");
        assert_eq!(output, input);
    }

    #[test]
    fn test_gui_roundtrip_position_startpos_moves() {
        let input = "position startpos moves 2g2f 8c8d 7g7f 3c3d\n";
        let msg = GuiMessage::parse(&input).unwrap();
        let output = format!("{msg}\n");
        assert_eq!(output, input);
    }

    #[test]
    fn test_gui_go() {
        let input = "\
        usinewgame
        position startpos moves 7g7f 3c3d
        go btime 300000 wtime 300000 byoyomi 5000
        ";

        let moves: Vec<Move> = vec![
            Move::BoardMove {
                from: Square::G7,
                to: Square::F7,
                promotion: false,
            },
            Move::BoardMove {
                from: Square::C3,
                to: Square::D3,
                promotion: false,
            },
        ];

        let params = EngineParams::new()
            .btime(Duration::from_millis(300000))
            .wtime(Duration::from_millis(300000))
            .byoyomi(Duration::from_millis(5000));
        let expect = vec![
            GuiMessage::UsiNewGame,
            GuiMessage::Position {
                sfen: None,
                moves: Some(moves),
            },
            GuiMessage::Go(params),
        ];
        let stream = GuiMessageStream::parse(input);
        for (parsed, expected) in stream.zip(expect) {
            assert_eq!(parsed, expected);
        }
    }

    //
    // Engine
    //

    #[test]
    fn test_engine_usiok() {
        let msg = EngineMessage::parse("usiok\n").unwrap();
        assert_eq!(msg, EngineMessage::UsiOk);
        assert_eq!(format!("{msg}\n"), "usiok\n");
    }

    #[test]
    fn test_engine_usiok_prefix() {
        // Note: `parse` only returns whatever is the first message, even if this is junk
        let msg = EngineMessage::parse("yoho usiok\n").unwrap();
        assert_eq!(msg, EngineMessage::Unknown(s("yoho ")));
    }

    #[test]
    fn test_engine_usiok_cr() {
        let msg = EngineMessage::parse("usiok\r").unwrap();
        assert_eq!(msg, EngineMessage::UsiOk);
    }

    #[test]
    fn test_engine_usiok_crnl() {
        let msg = EngineMessage::parse("usiok\r\n").unwrap();
        assert_eq!(msg, EngineMessage::UsiOk);
    }

    #[test]
    fn test_engine_first_valid() {
        // `parse_first_valid` skips all preceding junk
        let msg = EngineMessage::parse_first_valid("yoho\nhey usiok \n").unwrap();
        assert_eq!(msg, EngineMessage::UsiOk);
    }

    #[test]
    fn test_engine_roundtrip_usiok() {
        let msg = EngineMessage::UsiOk;
        let input = format!("{msg}\n");
        assert_eq!(EngineMessage::parse(&input).unwrap(), msg);
        assert_eq!(format!("{msg}\n"), input);
    }

    #[test]
    fn test_engine_id_name() {
        let input = "id name haitaka-shogi\n";
        let msg = EngineMessage::parse(input).unwrap();
        assert_eq!(msg, EngineMessage::Id(IdParams::Name(s("haitaka-shogi"))));
        assert_eq!(input, format!("{msg}\n"));
    }

    #[test]
    fn test_engine_id_author() {
        let input = "id author tofutofu\n";
        let msg = EngineMessage::parse(input).unwrap();
        assert_eq!(msg, EngineMessage::Id(IdParams::Author(s("tofutofu"))));
        assert_eq!(input, format!("{msg}\n"));
    }

    #[test]
    fn test_engine_option_check() {
        let input = "option name Nullmove type check default true\n";
        let msg = EngineMessage::parse(input).unwrap();
        assert_eq!(
            msg,
            EngineMessage::Option(OptionParam::Check {
                name: s("Nullmove"),
                default: Some(true)
            })
        );
        assert_eq!(input, format!("{msg}\n"));
    }

    #[test]
    fn test_engine_option_check_false() {
        let input = "option name USI_Ponder type check default false\n";
        let msg = EngineMessage::parse(input).unwrap();
        assert_eq!(
            msg,
            EngineMessage::Option(OptionParam::Check {
                name: s("USI_Ponder"),
                default: Some(false)
            })
        );
        assert_eq!(input, format!("{msg}\n"));
    }

    #[test]
    fn test_engine_option_spin() {
        let input = "option name Selectivity type spin default 2 min 0 max 4\n";
        let msg = EngineMessage::parse(input).unwrap();
        assert_eq!(
            msg,
            EngineMessage::Option(OptionParam::Spin {
                name: s("Selectivity"),
                default: Some(2),
                min: Some(0),
                max: Some(4)
            })
        );
        assert_eq!(input, format!("{msg}\n"));
    }

    #[test]
    fn test_engine_option_combo() {
        let input = "option name Style type combo default Normal var Solid var Normal var Wild\n";
        let msg = EngineMessage::parse(input).unwrap();
        assert_eq!(
            msg,
            EngineMessage::Option(OptionParam::Combo {
                name: s("Style"),
                default: Some(s("Normal")),
                vars: vec![s("Solid"), s("Normal"), s("Wild")]
            })
        );
        assert_eq!(input, format!("{msg}\n"));
    }

    #[test]
    fn test_engine_bestmove() {
        let input = "bestmove 3c3d\n";
        let msg = EngineMessage::parse(input).unwrap();
        let bestmove: Move = "3c3d".parse::<Move>().unwrap();
        let ponder: Option<Move> = None;
        assert_eq!(
            msg,
            EngineMessage::BestMove(BestMoveParams::BestMove { bestmove, ponder })
        );
        assert_eq!(input, format!("{msg}\n"));
    }

    #[test]
    fn test_engine_bestmove_ponder() {
        let input = "bestmove 8c8d ponder 3c3d\n";
        let msg = EngineMessage::parse(input).unwrap();
        let bestmove: Move = "8c8d".parse::<Move>().unwrap();
        let pondermove: Move = "3c3d".parse::<Move>().unwrap();
        let ponder: Option<Move> = Some(pondermove);
        assert_eq!(
            msg,
            EngineMessage::BestMove(BestMoveParams::BestMove { bestmove, ponder })
        );
        assert_eq!(input, format!("{msg}\n"));
    }

    #[test]
    fn test_engine_bestmove_resign() {
        let input = "bestmove resign\n";
        let msg = EngineMessage::parse(input).unwrap();
        assert_eq!(msg, EngineMessage::BestMove(BestMoveParams::Resign));
        assert_eq!(input, format!("{msg}\n"));
    }

    #[test]
    fn test_engine_bestmove_win() {
        let input = "bestmove win\n";
        let msg = EngineMessage::parse(input).unwrap();
        assert_eq!(msg, EngineMessage::BestMove(BestMoveParams::Win));
        assert_eq!(input, format!("{msg}\n"));
    }

    #[test]
    fn test_engine_info_currline() {
        let input = "info currline 2g2f 8c8d 7g7f\n";
        let line: Vec<Move> = vec![
            "2g2f".parse::<Move>().unwrap(),
            "8c8d".parse::<Move>().unwrap(),
            "7g7f".parse::<Move>().unwrap(),
        ];
        let msg = EngineMessage::parse(input).unwrap();
        assert_eq!(
            msg,
            EngineMessage::Info(vec![InfoParam::CurrLine {
                cpu_nr: None,
                line: line
            }])
        );
        assert_eq!(format!("{msg}\n"), input);
    }

    #[test]
    fn test_engine_info_currline_with_cpunr() {
        let input = "info currline 3 2g2f 8c8d 7g7f\n";
        let line: Vec<Move> = vec![
            "2g2f".parse::<Move>().unwrap(),
            "8c8d".parse::<Move>().unwrap(),
            "7g7f".parse::<Move>().unwrap(),
        ];
        let msg = EngineMessage::parse(input).unwrap();
        assert_eq!(
            msg,
            EngineMessage::Info(vec![InfoParam::CurrLine {
                cpu_nr: Some(3),
                line: line
            }])
        );
        assert_eq!(format!("{msg}\n"), input);
    }

    #[test]
    fn test_engine_message_stream1() {
        let input = "\
            id name haitaka-shogi
            id author tofutofu
            option name Nullmove type check default true
            option name Selectivity type spin default 2 min 0 max 4
            option name Style type combo default Normal var Solid var Normal var Wild
            option name USI_Ponder type check default false
            usiok
        ";
        let expect: Vec<EngineMessage> = vec![
            EngineMessage::Id(IdParams::Name(s("haitaka-shogi"))),
            EngineMessage::Id(IdParams::Author(s("tofutofu"))),
            EngineMessage::Option(OptionParam::Check {
                name: s("Nullmove"),
                default: Some(true),
            }),
            EngineMessage::Option(OptionParam::Spin {
                name: s("Selectivity"),
                default: Some(2),
                min: Some(0),
                max: Some(4),
            }),
            EngineMessage::Option(OptionParam::Combo {
                name: s("Style"),
                default: Some(s("Normal")),
                vars: vec![s("Solid"), s("Normal"), s("Wild")],
            }),
            EngineMessage::Option(OptionParam::Check {
                name: s("USI_Ponder"),
                default: Some(false),
            }),
            EngineMessage::UsiOk,
        ];
        let stream = EngineMessageStream::parse(input);
        for (parsed, expected) in stream.zip(expect) {
            assert_eq!(parsed, expected);
        }
    }

    #[test]
    fn test_engine_message_stream2() {
        let input = "\
        info depth 1 seldepth 0
        info nps 1234567
        info score cp 13  depth 1 nodes 13 time 15 pv 2g2f


        info currmove 2g2f currmovenumber 1
        info nodes 120000 nps 116391 hashfull 104
        info string 7g7f (70%)
        info score cp 156 multipv 1 pv P*5h 4g5g 5h5g 8b8f
        ";
        let expect: Vec<EngineMessage> = vec![
            EngineMessage::Info(vec![InfoParam::Depth(1), InfoParam::SelDepth(0)]),
            EngineMessage::Info(vec![InfoParam::Nps(1234567)]),
            EngineMessage::Info(vec![
                InfoParam::ScoreCp(13, ScoreBound::Exact),
                InfoParam::Depth(1),
                InfoParam::Nodes(13),
                InfoParam::Time(Duration::from_millis(15)),
                InfoParam::Pv(vec!["2g2f".parse::<Move>().unwrap()]),
            ]),
            EngineMessage::Info(vec![
                InfoParam::CurrMove("2g2f".parse::<Move>().unwrap()),
                InfoParam::CurrMoveNumber(1),
            ]),
            EngineMessage::Info(vec![
                InfoParam::Nodes(120_000),
                InfoParam::Nps(116391),
                InfoParam::HashFull(104),
            ]),
            EngineMessage::Info(vec![InfoParam::String(s("7g7f (70%)"))]),
            EngineMessage::Info(vec![
                InfoParam::ScoreCp(156, ScoreBound::Exact),
                InfoParam::MultiPv(1),
                InfoParam::Pv(vec![
                    "P*5h".parse::<Move>().unwrap(),
                    "4g5g".parse::<Move>().unwrap(),
                    "5h5g".parse::<Move>().unwrap(),
                    "8b8f".parse::<Move>().unwrap(),
                ]),
            ]),
        ];
        let stream = EngineMessageStream::parse(input);
        for (parsed, expected) in stream.zip(expect) {
            assert_eq!(parsed, expected);
        }
    }
}
