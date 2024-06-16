# Random tag

Command-line utility for generating random tags.

[![Random tag crate](https://img.shields.io/crates/v/random-tag.svg)](https://crates.io/crates/random-tag)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
![minimum rustc 1.74](https://img.shields.io/badge/rustc-1.74+-red.svg)

## Installation

Crate requires `rustc 1.74.0` or greater.

``` bash
cargo install random-tag
```

## Usage

```
Generate random tags

Usage: random-tag [OPTIONS] [COMMAND]

Commands:
  completions  Outputs the completion file for given shell
  dump-tags    Dump used tags as CSV to stdout
  load-tags    Load used tags from stdin or file in CSV format
  check-db     Check used tags database
  drop-db      Drop used tags database
  help         Print this message or the help of the given subcommand(s)

Options:
  -c, --chars <CHARS>    Tag chars [default: dfqsvz0123456789]
  -l, --length <LENGTH>  Tag length from 1 to 255 [default: 6]
  -a, --amount <AMOUNT>  Tags amount from 1 to 255 [default: 10]
  -h, --help             Print help
  -V, --version          Print version
```
