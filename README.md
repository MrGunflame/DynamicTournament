# DynamicTournament

## Building (Web)

### Requirements (build only)

- A stable rust toolchain
- [Trunk](https://crates.io/crates/trunk) (`cargo install trunk`)

### Building

- Clone the repo: `git clone https://github.com/MrGunflame/dynamic-tournament`
- Build using trunk: `trunk build --release`

All bundled files can be found in `dist/`.

## Building (Server)

### Requirements

- A stable rust toolchain (build only).

### Building

- Clone the repo: `git clone https://github.com/MrGunflame/dynamic-tournament`
- Build the server: `cargo build --bin dynamic-tournament-server --release`

The final binary will be `target/release/dynamic-tournament-server`.

For more information see the server crate: https://github.com/MrGunflame/DynamicTournament/tree/master/dynamic-tournament-server

## License

This project is licensed under the [Apache License, Version 2.0](https://github.com/MrGunflame/DynamicTournament/blob/master/LICENSE) unless otherwise stated.
