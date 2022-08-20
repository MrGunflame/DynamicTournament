SHELL := /bin/bash

PATH_ROOT := $(shell git rev-parse --show-toplevel)

PATH_BUILD := $(PATH_ROOT)/build
PATH_BIN := $(PATH_BUILD)/bin
PATH_DIST := $(PATH_BUILD)/dist

RUSTUP := rustup
CARGO := cargo

.PHONY: build trunk toolchain docker

all: build

trunk:
	@if [ ! -f "$(PATH_BUILD)/bin/trunk" ]; \
	then \
		$(CARGO) install trunk --root $(PATH_BUILD); \
	fi

build: trunk toolchain
	$(RUSTUP) toolchain install stable
	$(RUSTUP) target add wasm32-unknown-unknown
	cd $(PATH_ROOT) && $(PATH_BIN)/trunk build --release --dist $(PATH_DIST)

docker:
	cd $(PATH_ROOT) && docker build --rm -t dynamic-tournament-web -f dynamic-tournament-web/Dockerfile .
