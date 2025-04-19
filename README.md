# haitaka-usi ハイタカ-usi

Haitaka-usi is a [Universal Shogi Interface (USI) protocol](http://hgm.nubati.net/usi.html) parser and serializer.

The USI protocol is a simple protocol for Shogi GUI clients and engines to communicate with each other. 
The protocol is based on the [UCI (Universal Chess Interface) protocol](https://backscattering.de/chess/uci/) for Chess engines with modifications to support Shogi.

The parser code uses a [PEST](https://github.com/pest-parser/pest) grammar to define the protocol syntax. The `usi.pest` grammar file
was initially inspired by the `uci.pest` file provided by [Vampirc UCI](https://github.com/vampirc/vampirc-uci/tree/master). The `uci.pest` file was a great help to get up to speed with using PEST, but in the end I decided to rewrite most of the grammar, both in order to make the grammar more uniform and to support USI features. The object model in `usi.rs` was also inspired by Vampirc UCI, but the actual implementation here is rather different. 

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
