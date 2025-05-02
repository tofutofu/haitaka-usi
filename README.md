# haitaka-usi ハイタカ-usi

Haitaka-usi is a [Universal Shogi Interface (USI) protocol](http://hgm.nubati.net/usi.html) parser and serializer.

The USI protocol is a simple, 7-bit-ascii-text-based protocol for Shogi clients and engines to communicate with each other. 
The protocol is based on the [UCI (Universal Chess Interface) protocol](https://backscattering.de/chess/uci/) with modifications to support Shogi.

The parser code in this repo uses a [PEG/PEST](https://github.com/pest-parser/pest) grammar to define the protocol syntax. 
The grammar file, `usi.pest`, was originally inspired by the the `uci.pest` file of [Vampirc UCI](https://github.com/vampirc/vampirc-uci/tree/master), which was a great help in getting up to speed with PEST. Apart from making the required modifications to support Shogi, however, I also made some other pretty significant changes to that base grammar. In particular, I unified the top-level entry points (the `start` rule) which should make it easier to use and verify the grammar. I also clearly separated GUI messages from Engine messages, and I made some other, smaller modifications in order to conform a tiny bit more closely to the spec. See comments in the `usi.pest` file for details.

This library implements serialization and deserialization for both the client (GUI) and server (engine) side. This makes it possible to use it both as frontend of a Shogi engine and as backend of a GUI client, so it can be an actual bridge between server and client. It differs in this regard from the [usi-rs](https://github.com/nozaq/usi-rs) crate which only provides serialization of GUI commands and parsing only of Engine commands.  

## Installation

TODO

## Usage

TODO

## Testing

TODO

## API

The API docs are available at TODO.

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


## References

- [将棋所USIプロトコル](https://shogidokoro2.stars.ne.jp/usi.html)
- [The Universal Shogi Interface](http://hgm.nubati.net/usi.html)
- [UCI protocol](https://backscattering.de/chess/uci/)
