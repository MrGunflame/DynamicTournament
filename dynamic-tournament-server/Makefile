SHELL := /bin/bash

CARGO := cargo

.PHONY: test build docker

all: build

test:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --all-targets --all-features -- -D warnings
	RUSTDOCFLAGS="-Dwarnings" $(CARGO) doc --no-deps --all-features
	$(CARGO) test --all-features

build:
	$(CARGO) build --release
	@printf "\nOUTPUT FILE PATH: \e[92m../target/release/dynamic-tournament-server\e[0m\n"

docker:
	cd .. && docker build --rm -t dynamic-tournament-server -f dynamic-tournament-server/Dockerfile .
