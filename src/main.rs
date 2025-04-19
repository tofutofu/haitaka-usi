use std::io::{self, Read, Write};
use haitaka_usi::parser::*;

fn main() {

    loop {

        print!(">>> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_to_string(&mut input).expect("Failed to read from stdin");

        if input == "quit" {
            break;
        }
    
        dbg(&input);

        let msg = parse_one(&input);

        println!("\n{:#?}", msg);

    }


}

/*

>>> go depth 3
Parsed successfully: 
[Pair { rule: go, span: Span { str: "go depth 3", start: 0, end: 10 }, 
    inner: [Pair { rule: go_full, span: Span { str: "go depth 3", start: 0, end: 10 }, 
        inner: [Pair { rule: go_search, span: Span { str: "depth 3", start: 3, end: 10 }, 
            inner: [Pair { rule: depth, span: Span { str: "depth 3", start: 3, end: 10 }, 
                inner: [Pair { rule: digits3, span: Span { str: "3", start: 9, end: 10 }, 
                    inner: [Pair { rule: digit, span: Span { str: "3", start: 9, end: 10 }, 
                        inner: [] }] }] }] }] }] }]

*/