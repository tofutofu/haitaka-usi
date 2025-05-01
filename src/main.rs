//use haitaka_usi::parser::*;
//use std::io::{self, Read, Write};

fn main() {
    println!("Hello USI");
}

/*
fn test() {
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

        let ml = parse(&input);

        //println!("\n{:#?}", msg);
        //println!("\nmsg='{}'", msg);

        for (i, m) in ml.iter().enumerate() {
            println!("[{}] '{}'", i, m);
        }
    }
}
*/
