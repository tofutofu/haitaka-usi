use haitaka_usi::parser::*;
//use haitaka_usi::gui::*;
//use haitaka_usi::engine::*;
use std::io::{self, Read, Write};

fn main() {
    println!("Hello USI");

    loop {
        print!(">>> ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin()
            .read_to_string(&mut input)
            .expect("Failed to read from stdin");
        if input == "quit" {
            break;
        }
        println!("Input: {:?}", input);
        println!("Debug:");
        dbg(&input);
        println!();

        let s = GuiMessageStream::new(&input).expect("Failed to parse stream");

        for msg in s {
            println!("{:?}", msg);
        }


        /* 
        let gui_msg = GuiMessage::parse(&input);

        if gui_msg.is_ok() {
            println!("GuiMessage: {}", gui_msg.unwrap());
        }
        else {
            println!("Gui Error: {}", gui_msg.err().unwrap());
        }

        let eng_msg = EngineMessage::parse(&input);
        if eng_msg.is_ok() {
            println!("EngineMessage: {}", eng_msg.unwrap());
        } else {
            println!("Eng err: {}", eng_msg.err().unwrap());
        }
        */
    }
}
