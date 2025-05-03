#[cfg(test)]
mod tests {
    use crate::*;
    use haitaka_types::Move;

    fn s(s: &str) -> String { s.to_owned() }

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
    fn test_gui_first_valid_missing_newline() {
        let result = std::panic::catch_unwind(|| {
            GuiMessage::parse_first_valid("yoho\nhey usi");
        });
        assert!(result.is_err(), "Expected a panic attack (missing newline), but none occurred");
    }

    //
    // roundtrip tests
    //

    #[test]
    fn test_gui_roundtrip_usi() {
        let msg = GuiMessage::Usi;
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_usinewgame() {
        let msg = GuiMessage::UsiNewGame;
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_debug_on() {
        let msg = GuiMessage::Debug(true);
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_debug_off() {
        let msg = GuiMessage::Debug(false);
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_isready() {
        let msg = GuiMessage::IsReady;
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_setoption_with_value() {
        let msg = GuiMessage::SetOption {
            name: s("USI_Hash"),
            value: Some(s("128")),
        };
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_setoption_without_value() {
        let msg = GuiMessage::SetOption {
            name: s("USI_Ponder"),
            value: None,
        };
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_register_later() {
        let msg = GuiMessage::Register {
            name: None,
            code: None,
        };
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_register_with_name_and_code() {
        let msg = GuiMessage::Register {
            name: Some(s("Fee Fie Foo")),
            code: Some(s("123 x 456")),
        };
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_stop() {
        let msg = GuiMessage::Stop;
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_quit() {
        let msg = GuiMessage::Quit;
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_gui_roundtrip_gameover() {
        let msg = GuiMessage::GameOver(GameStatus::Win);
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);

        let msg = GuiMessage::GameOver(GameStatus::Lose);
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);

        let msg = GuiMessage::GameOver(GameStatus::Draw);
        let s = format!("{msg}\n");
        assert_eq!(GuiMessage::parse(&s).unwrap(), msg);
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
        assert_eq!(input, output);
    }

    #[test]
    fn test_gui_roundtrip_position_startpos_moves() {
        let input = "position startpos moves 2g2f 8c8d 7g7f 3c3d\n";
        let msg = GuiMessage::parse(&input).unwrap();
        let output = format!("{msg}\n");
        assert_eq!(input, output);
    }

    //
    // Engine
    //

    #[test]
    fn test_engine_usiok() {
        let msg = EngineMessage::parse("usiok\n").unwrap();
        assert_eq!(msg, EngineMessage::UsiOk);
    }

    #[test]
    fn test_engine_usiok_prefix() {
        let msg = EngineMessage::parse("yoho usiok\n").unwrap();
        assert_eq!(msg, EngineMessage::Unknown(s("yoho ")));
    }

    #[test]
    fn test_engine_first_valid() {
        let msg = EngineMessage::parse_first_valid("yoho\nhey usiok \n").unwrap();
        assert_eq!(msg, EngineMessage::UsiOk);
    }

    #[test]
    fn test_engine_roundtrip_usiok() {
        let msg = EngineMessage::UsiOk;
        let s = format!("{msg}\n");
        assert_eq!(EngineMessage::parse(&s).unwrap(), msg);
    }

    #[test]
    fn test_engine_message_stream() {
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
            EngineMessage::Option(OptionParam::Check { name: s("Nullmove"), default: Some(true) }),
            EngineMessage::Option(OptionParam::Spin { name: s("Selectivity"), default: Some(2), min: Some(0), max: Some(4)}),
            EngineMessage::Option(OptionParam::Combo { name: s("Style"), default: Some(s("Normal")),
                vars: vec![s("Solid"), s("Normal"), s("Wild")]}),
            EngineMessage::Option(OptionParam::Check { name: s("USI_Ponder"), default: Some(false) }),
            EngineMessage::UsiOk,
        ];
        let stream = EngineMessageStream::parse(input);
        for (parsed, expected) in stream.zip(expect) {
            assert_eq!(parsed, expected);
        }
    }


    /*
            let ml = parse("usi\nusi\n");
            assert_eq!(ml.len(), 2);
            for mb in ml {
                assert_eq!(mb, UsiMessage::UsiGuiToEngine(GuiMessage::Usi));
            }

    // Option examples
    // ---------------
    // option name Nullmove type check default true\n
    // option name Selectivity type spin default 2 min 0 max 4\n
    // option name Style type combo default Normal var Solid var Normal var Risky\n
    // option name NalimovPath type string default c:\\n
    // option name Clear Hash type button\n
    //
    // "Certain options have a fixed value for id, which means that the semantics of this option is fixed."
    // For Shogi those have prefix "USI_" rather than "UCI_".

    // Info examples
    // -------------
    //  info currmove e2e4 currmovenumber 1
    // 	info depth 12 nodes 123456 nps 100000
    //
    // The spec says that "all infos belonging to the pv should be sent together", e.g.
    //
    //  info depth 2 score cp 214 time 1242 nodes 2124 nps 34928 pv 2g2f 8c8d 2f2e 8d8e
    //
    // however, it forgets to mention which infos always "belong to" the pv.
    // The 'pv' apparently always ends an info command, so it can also not go togeter with the
    // 'info string ...' message (Shogidokoro).
    //
    //  info depth 1 seldepth 0
    //  info score cp 13  depth 1 nodes 13 time 15 pv 2g2f
    //  info depth 2 seldepth 2
    //  info nps 15937
    //  info score cp 14  depth 2 nodes 255 time 15 pv 2g2f 8c8d
    //  info depth 2 seldepth 7 nodes 255
    //  info depth 3 seldepth 7
    //  info nps 26437
    //  info score cp 20  depth 3 nodes 423 time 15 pv 2g2f P*8f
    //  info nps 41562


            */
}
