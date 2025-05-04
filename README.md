# haitaka-usi ハイタカ-usi &emsp; [![Latest Version]][crates.io] [![Build Status]][actions] [![Documentation]][docs]

[Build Status]: https://img.shields.io/github/actions/workflow/status/tofutofu/haitaka-usi/rust.yml?branch=main
[actions]: https://github.com/tofutofu/haitaka-usi/actions?query=branch%3Amain
[Latest Version]: https://img.shields.io/crates/v/haitaka-usi.svg
[crates.io]: https://crates.io/crates/haitaka-usi
[Documentation]: https://docs.rs/haitaka-usi/badge.svg
[docs]: https://docs.rs/haitaka-usi

Haitaka-usi is a [Universal Shogi Interface (USI) protocol](http://hgm.nubati.net/usi.html) parser and serializer.

The USI protocol is a simple, 7-bit-ascii-text-based protocol for Shogi clients and engines to communicate with each other. 
The protocol is based on the [UCI (Universal Chess Interface) protocol](https://backscattering.de/chess/uci/) with modifications to support Shogi.

The parser code in this repo uses a [PEG/PEST](https://github.com/pest-parser/pest) grammar to define the protocol syntax. The grammar file, [`usi.pest`](https://github.com/tofutofu/haitaka-usi/blob/main/src/usi.pest), was originally inspired by the `uci.pest` file of [Vampirc UCI](https://github.com/vampirc/vampirc-uci/tree/master), which was a great help in getting up to speed with PEST. But apart from making the required modifications to support Shogi, I also made some other pretty significant changes. In particular, I unified the top-level entry points (the `start` rule) which should make it easier to use and verify the grammar. I also clearly separated GUI messages from Engine messages, and I made a few other, smaller modifications to conform more closely to the spec. See comments in the `usi.pest` file for details. 

This library implements serialization and deserialization for both the client (GUI) and server (engine) side. This makes it possible to use as a bridge between a server (Shogi engine frontend) and a client (GUI backend). It differs in this regard from the [usi-rs](https://github.com/nozaq/usi-rs) crate which is an alternative implementation and an excellent GUI client, but which only supports deserialization of GUI commands and parsing of Engine commands.

## Installation

Add `haitaka-usi` to your `Cargo.toml`:
```toml
[dependencies]
haitaka-usi = "0.1.0"   # or use the latest version on crates.io
```

## Usage

### Deserialization

```rust
use haitaka_types::*;
use haitaka_usi::*;

// parse a multi-line sequence of messages
let input = "\
    id name haitaka-shogi
    id author tofutofu
    option name Style type combo default Normal var Solid var Normal var Wild
    option name USI_Hash type spin default 256
    option name USI_Ponder type check default false
    usiok
";
for msg in EngineMessageStream::new(input) {
    println!("{msg:?}");
    println!("{msg}");
}

// parse one line
let input = "bestmove 2g2f ponder 8c8d\n";
let bestmove = Move::BoardMove { from: Square::G2, to: Square::F2, promotion: false };
let ponder = Some(Move::BoardMove { from: Square::C8, to: Square::D8, promotion: false });

let msg = EngineMessage::parse(input).unwrap();
assert_eq!(
    msg, 
    EngineMessage::BestMove(BestMoveParams::BestMove { bestmove, ponder })
);

// parse one line, skipping junk tokens
let input = "yo taka usiok\n";
let msg = EngineMessage::parse_first_valid(input).unwrap();
assert_eq!(msg, EngineMessage::UsiOk);

// parse a gui message multi-line input string
let input = "\
    usinewgame
    position startpos moves 7g7f 3c3d
    go btime 300000 wtime 300000 byoyomi 10000
";
for msg in GuiMessageStream::new(input) {
    println!("{msg:?}");
    println!("{msg}");
}

// parse one line
let input = "usinewgame\n";
let msg = GuiMessage::parse(input).unwrap();
assert_eq!(msg, GuiMessage::UsiNewGame);
```

## Serialization

```rust
use haitaka_types::*;
use haitaka_usi::*;

let params = EngineParams::new().btime(300000).wtime(300000).byoyomi(10000);
let msg = GuiMessage::Go(params);

assert_eq!(format!("{msg}\n"), "go btime 300000 wtime 300000 byoyomi 10000\n"); 

let bestmove = Move::BoardMove { from: Square::G2, to: Square::F2, promotion: false };
let ponder = Some(Move::BoardMove { from: Square::C8, to: Square::D8, promotion: false });
let msg = EngineMessage::BestMove(BestMoveParams::BestMove { bestmove, ponder });

assert_eq!(format!("{msg}\n"), "bestmove 2g2f ponder 8c8d\n");
```

More examples can be found in the [unit tests](https://github.com/tofutofu/haitaka-usi/blob/main/src/tests.rs).

## API

The API docs will be available at [docs.rs/haitaka-usi](https://docs.rs/haitaka-usi).

## Current limitations

The main limitation is that the current library does not support a real streaming protocol. All parser functions
require a complete input string. `GuiMessageStream` and `EngineMessageStream` can convert a given input string
into a sequence of message objects, but also require the input to be fully given. 

## Supported Protocol Messages

### GuiToEngine
```text
    usi
    usinewgame
    isready
    debug
    setoption
    register
    ponderhit
    position 
    go
    stop
    quit
    gameover
```

### EngineToGui
```text
    id
    usiok
    copyprotection
    registration 
    option
    readyok
    bestmove
    info
```

## Contributing

Contributions are very welcome! Please open an issue or submit a pull request on GitHub.
Also, feel free to contact me on the Discord Shogi Harbour channel (@tofutofu).

## License

`haitaka-usi` is licensed under the [MIT license](https://github.com/tofutofu/haitaka-usi/blob/main/LICENSE).

## References

- [将棋所USIプロトコル](https://shogidokoro2.stars.ne.jp/usi.html)
- [The Universal Shogi Interface](http://hgm.nubati.net/usi.html)
- [UCI protocol](https://backscattering.de/chess/uci/)
