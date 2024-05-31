# vlcb-network

TODO: change paths
[![docs.rs](https://docs.rs/smoltcp/badge.svg)](https://docs.rs/smoltcp)
[![crates.io](https://img.shields.io/crates/v/smoltcp.svg)](https://crates.io/crates/smoltcp)
[![crates.io](https://img.shields.io/crates/d/smoltcp.svg)](https://crates.io/crates/smoltcp)
[![crates.io](https://img.shields.io/matrix/smoltcp:matrix.org)](https://matrix.to/#/#smoltcp:matrix.org)
[![codecov](https://codecov.io/github/smoltcp-rs/smoltcp/branch/master/graph/badge.svg?token=3KbAR9xH1t)](https://codecov.io/github/smoltcp-rs/smoltcp)

_vlcb-network_ is a standalone, event-driven TCP/IP stack that is designed for bare-metal,
real-time systems. Its design goals are simplicity and robustness. Its design anti-goals
include complicated compile-time computations, such as macro or type tricks, even
at cost of performance degradation.

_vlcb-network_ does not need heap allocation *at all*, is [extensively documented][docs],
and compiles on stable Rust 1.65 and later.

[docs]: https://docs.rs/smoltcp/

## Features

_vlcb-network_ is missing many widely deployed features, usually because no one implemented them yet.
To set expectations right, both implemented and omitted features are listed.

### Media layer

* CAN
  * Standard frames are supported (extended are dropped at the moment).

### VLCB layer

VLCB does not specify any "sub-protocols" but this library does separate certain traffics
where it's more convenient to do so.

#### Module

Module socket is currently used for all the traffic except the long message packets.

#### Long Message

TODO

## Installation

TBA

## Feature flags

### Feature `myfeature`

TBA

## Configuration
TBA

## License

_vlcb-network_ is distributed under the terms of Apache 2.0 license.

See [LICENSE](LICENSE) for details.