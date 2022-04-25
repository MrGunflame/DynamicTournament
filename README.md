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

## TODO

These features are still work in progress:
- [ ] Different Tournament Types (e.g. Double Elimination, Round-robin, Swiss, etc..)
- [x] A way to save and load the bracket state
- [x] A proper score system
- [x] Auto-forward placeholder matches
- [ ] A way to undo matches and revert the bracket state
- [x] DoubleElimination
- [ ] Seeding
- [ ] Full refactor for v1.0
- [ ] Live bracket updates with WebSockets
- [ ] Split Entrants and Data in dynamic-tournament-generator
