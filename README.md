# Random tag
Command-line utility for generating random tags

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/random-tag.svg
[crates-url]: https://crates.io/crates/random-tag
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE

# Installation
``` bash
cargo install random-tag
```

# Usage
```
random-tag 0.1.4
Generate random tags

USAGE:
    random-tag [OPTIONS]

OPTIONS:
    -a, --amount <AMOUNT>    Tags amount from 1 to 255 [default: 1]
    -c, --chars <CHARS>      Tag chars [default: dfqsvz0123456789]
    -h, --help               Print help information
    -l, --length <LENGTH>    Tag length from 1 to 255 [default: 6]
    -V, --version            Print version information
```
