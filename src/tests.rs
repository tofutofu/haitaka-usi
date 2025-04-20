#[cfg(test)]
mod tests {
    //use std::io::*;
    use crate::*;

    #[test]
    fn test_usi() {
        let ml = parse("usi\nusi\n");
        assert_eq!(ml.len(), 2);
        for mb in ml {
            assert_eq!(mb, UsiMessage::UsiGuiToEngine(GuiMessage::Usi));
        }
    }
}

