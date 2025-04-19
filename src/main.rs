use haitaka_usi::parser::*;
use std::io::{self, Read, Write};

fn main() {
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

        dbg(&input);

        let msg = parse_one(&input);

        println!("\n{:#?}", msg);
        println!("\nmsg='{}'", msg);
    }
}
