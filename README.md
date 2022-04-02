# Random tag
Command-line utility for generating random tags.

[![Random tag crate](https://img.shields.io/crates/v/random-tag.svg)](https://crates.io/crates/random-tag)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![minimum rustc 1.59](https://img.shields.io/badge/rustc-1.59+-red.svg)

## Installation
Crate requires `rustc 1.59.0` or greater. 
``` bash
cargo install random-tag
```

## Usage
```
random-tag 0.1.11
Gleb Chipiga
Generate random tags

USAGE:
    random-tag [OPTIONS]

OPTIONS:
    -a, --amount <AMOUNT>    Tags amount from 1 to 255 [default: 1]
    -c, --chars <CHARS>      Tag chars [default: dfqsvz0123456789]
    -h, --help               Print help information
    -l, --length <LENGTH>    Tag length from 1 to 255 [default: 6]
    -s, --shell <SHELL>      Outputs the completion file for given shell [possible values: bash,
                             elvish, fish, powershell, zsh]
    -V, --version            Print version information
```
