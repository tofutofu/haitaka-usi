#[cfg(test)]
mod tests {
    //use std::io::*;
    // use crate::*;

    #[test]
    fn test_usi() {
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
}
