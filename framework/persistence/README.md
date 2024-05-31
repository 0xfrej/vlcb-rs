# vlcb-persistence

[![docs.rs](https://docs.rs/vlcb-persistence/badge.svg)](https://docs.rs/vlcb-persistence)
[![crates.io](https://img.shields.io/crates/v/vlcb-persistence.svg)](https://crates.io/crates/vlcb-persistence)
[![crates.io](https://img.shields.io/crates/d/vlcb-persistence.svg)](https://crates.io/crates/vlcb-persistence)
[![crates.io](https://img.shields.io/matrix/vlcb-persistence:matrix.org)](https://matrix.to/#/#vlcb-persistence:matrix.org)

_vlcb-persistence_ is implementation of node config storage abstraction for persisting values such as learned events,
node variables, the node number and others. The implementation is built on top of the `embedded-storage` trait crate
to ensure driver availability for the end-users and easy implementation of missing drivers whenever needed.

This crate builds on `no_std` and does not require heap allocation.

## Features

Currently the only actual use of this crate is the node config storage but in future we may add
other uses. As such this crate defines some base traits that all specific storage implementations should follow
for behavioral reasons (such as ability to wipe/flush the storage and etc).

### Storage implementations

- NodeConfig - Node configuration abstraction
  - NodeConfigStorage - in-memory storage implementation
  - PersistentNodeConfigStorage - cached storage implementation backed up by persistent storage by a `embedded-storage` driver

## License

_vlcb-network_ is distributed under the terms of Apache 2.0 license.

See [LICENSE](LICENSE) for details.