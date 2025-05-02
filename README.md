# haitaka-usi ハイタカ-usi

Haitaka-usi is a [Universal Shogi Interface (USI) protocol](http://hgm.nubati.net/usi.html) parser and serializer.

The USI protocol is a simple, 7-bit-ascii-text based protocol for Shogi GUI clients and engines to communicate with each other. 
The protocol is based on the [UCI (Universal Chess Interface) protocol](https://backscattering.de/chess/uci/) with modifications to support Shogi.

The parser code uses a [PEST](https://github.com/pest-parser/pest) grammar to define the protocol syntax.

The grammar file, `usi.pest`, was inspired by the the `uci.pest` file of [Vampirc UCI](https://github.com/vampirc/vampirc-uci/tree/master), which was a great help in getting up to speed with PEST, but apart from modifications to support Shogi, I also tried to make the grammar rules more uniform and streamlined.

The code implements serialization and deserialization for both the GUI and the Engine-side. It differs in this regard from the [usi-rs](https://github.com/nozaq/usi-rs) crate that provides serialization only of GUI commands and parsing only for Engine commands.

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

- The Universal Shogi Interface (http://hgm.nubati.net/usi.html)
- UCI protocol (https://backscattering.de/chess/uci/)
