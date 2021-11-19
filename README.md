# Rust IEEE 802.15.4 [![crates.io](https://img.shields.io/crates/v/ieee802154.svg)](https://crates.io/crates/ieee802154) [![Documentation](https://docs.rs/ieee802154/badge.svg)](https://docs.rs/ieee802154) [![Build Status](https://github.com/rust-iot/rust-ieee802.15.4/actions/workflows/run-test.yml/badge.svg)](https://github.com/rust-iot/rust-ieee802.15.4/actions/workflows/run-test.yml)

## Introduction

Partial [Rust] implementation of the [IEEE 802.15.4] standard, which defines the operation of low-rate wireless personal area networks. This crate is in early development and only implements a small subset of IEEE 802.15.4.

[Rust]: https://www.rust-lang.org/
[IEEE 802.15.4]: https://en.wikipedia.org/wiki/IEEE_802.15.4


## Usage

Use Cargo to add this library as a dependency to your project. Add the following to you `Cargo.toml`:
``` toml
[dependencies]
ieee802154 = "0.5"
```

For more information, please refer to the [API Reference].

[API Reference]: https://docs.rs/ieee802154


## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][] (0BSD, for short). This basically means you can do anything with the software, without any restrictions, but you can't hold the authors liable for problems.

See [LICENSE] for full details.

[Zero Clause BSD License]: https://opensource.org/licenses/FPL-1.0.0
[LICENSE]: https://github.com/braun-embedded/rust-ieee802.15.4/blob/master/LICENSE


**Created by [Braun Embedded](https://braun-embedded.com/)** <br />
**Initial development sponsored by [Ferrous Systems](https://ferrous-systems.com/)**
