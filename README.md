# DynamicTournament

A modular tournament generator and viewer.

## Crates

The project is split up in a modular approach. Components are connected via the API.

- `dynamic-tournament-web`: A WebAssembly frontend for viewing tournaments.
- `dynamic-tournament-server`: A server implementation of the tournaments API.
- `dynamic-tournament-api`: Contains API types and an API client capable of using the system or WebAssembly interface.
- `dynamic-tournament-core`: The point of this project. Contains generic types and functions for creating and rendering tournament trees.
- `dynamic-tournament-macros`: Shared proc macros for the `dynamic-tournament-web` and `dynamic-tournament-server` crates.
- `dynamic-tournament-test`: A utility crate for generating test data for the API.
- `dynamic-tournament-cli`: A CLI for interacting with the API.

## Building

### Requirements

- A stable rust toolchain with `rustup` and `cargo`.
- Docker if building the server docker image.

### Web Client

- Clone the repo: `git clone https://github.com/MrGunflame/DynamicTournament`
- Build with make: `cd DynamicTournament/dynamic-tournament-web && make`

See [dynamic-tournament-web](https://github.com/MrGunflame/DynamicTournament/tree/master/dynamic-tournament-web) for more details.

### Server

- Clone the repo: `git clone https://github.com/MrGunflame/DynamicTournament`
- Build with make: `cd DynamicTournament/dynamic-tournament-server && make`

See [dynamic-tournament-server](https://github.com/MrGunflame/DynamicTournament/tree/master/dynamic-tournament-server) for more details.

## Documentation

The API documentation for all currently stable versions [can be found here](https://github.com/MrGunflame/DynamicTournament/tree/master/docs).

The documentation for all rust crates must be generated manually:
- `cargo doc --no-deps`

## License

This project is licensed under the [Apache License, Version 2.0](https://github.com/MrGunflame/DynamicTournament/blob/master/LICENSE) unless otherwise stated.
