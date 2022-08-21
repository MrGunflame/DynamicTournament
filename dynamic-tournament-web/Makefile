SHELL := /bin/bash

PATH_ROOT := $(shell git rev-parse --show-toplevel)
PATH_PKG := $(PATH_ROOT)/dynamic-tournament-web

PATH_BUILD := $(PATH_ROOT)/build
PATH_BIN := $(PATH_BUILD)/bin
PATH_DIST := $(PATH_BUILD)/dist

RUSTUP := rustup
CARGO := cargo

.PHONY: wasm-pack build prepare-hash hash docker clean

all: build hash

wasm-pack:
	@if [ ! -f "$(PATH_BUILD)/bin/wasm-pack" ]; \
	then \
		$(CARGO) install wasm-pack --root $(PATH_BUILD); \
	fi

build: wasm-pack
	# Clean old files
	rm -rf $(PATH_DIST)

	$(RUSTUP) toolchain install stable
	$(RUSTUP) target add wasm32-unknown-unknown
	cd $(PATH_ROOT) && $(PATH_BIN)/wasm-pack build --release --target web --no-typescript --out-dir $(PATH_DIST) $(PATH_PKG)

prepare-hash:
	@for path in find sed awk; \
	do \
		which $$path > /dev/null; \
		if [ $$? -ne 0 ]; \
		then \
			echo "Cannot find $$path"; \
			exit 1; \
		fi \
	done

hash: prepare-hash build
	@for file in $$(find $(PATH_DIST) -regextype sed -regex '^.*\.\(wasm\|css\|js\)$$'); \
	do \
		DIR=$$(ls $$file | sed 's/[^\/]*$$//'); \
		HASH=$$(sha256sum $$file | awk '{ print $$1 }' | head -c 20); \
		SUFFIX=$$(ls $$file | awk -F '.' '{ print $$NF }'); \
		\
		DST="$$DIR/$$HASH.$$SUFFIX"; \
		mv -f $$file $$DST; \
		echo "Moved $$file to $$DST"; \
	done

	# Update the wasm blob path in the js file.
	WASM_FILE=$$(find $(PATH_DIST) -regextype sed -regex '.*\.wasm' | awk -F '/' '{ print $$NF }'); \
	JS_FILE=$$(find $(PATH_DIST) -regextype sed -regex '.*\.js'); \
	sed -i "s/dynamic_tournament_web_bg.wasm/$$WASM_FILE/" $$JS_FILE


docker:
	cd $(PATH_ROOT) && docker build --rm -t dynamic-tournament-web -f dynamic-tournament-web/Dockerfile .

clean:
	rm -rf $(PATH_BUILD)
