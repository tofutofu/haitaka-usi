# haitaka-usi ハイタカ-USI

Haitaka-usi is a [Universal Shogi Interface (USI) protocol](http://hgm.nubati.net/usi.html) parser and serializer.

The USI protocol is a simple protocol for Shogi GUI clients and engines to communicate with each other. 
The protocol is derived from the [UCI (Universal Chess Interface) protocol](https://backscattering.de/chess/uci/) for Chess engines, with minor modifications to support Shogi.

The parser code used a [PEST](https://github.com/pest-parser/pest) grammar to define the protocol syntax. The `usi.pest` grammar file
was initially inspired by the `uci.pest` file provided by [Vampirc UCI](https://github.com/vampirc/vampirc-uci/tree/master). The `uci.pest` file was a great help to quickly get up to speed with using PEST, but in the end I decided to rewrite most of the grammar in order to support the required changes for Shogi and to streamline the grammar.


## Installation

TODO

## Usage

TODO

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
