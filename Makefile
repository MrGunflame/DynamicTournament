SHELL := /bin/bash

CARGO := cargo

BUILD_PATH := $(shell pwd)/build
PATH_BIN := $(BUILD_PATH)/bin
PATH_DIST := $(BUILD_PATH)/dist

MINIFIER := $(PATH_BIN)/minifier
MINIFIER_ARGS := --do-not-minify-doctype --ensure-spec-compliant-unquoted-attribute-values --keep-spaces-between-attributes --minify-css  --minify-js

.PHONY: test prerequisites build trunk setup-minify minify

all: prerequisites build minify

test:
	$(CARGO) fmt --all -- --check
	$(CARGO) clippy --all-targets --all-features -- -D warnings
	RUSTDOCFLAGS="-Dwarnings" $(CARGO) doc --no-deps --all-features
	$(CARGO) test --all-features

prerequisites:
	@for path in git cargo go; \
	do \
		which $$path > /dev/null; \
		if [ $$? -ne 0 ]; \
		then \
			echo "$$path is not installed"; \
			exit 1; \
		fi \
	done

trunk: 
	@if [ ! -f "$(BUILD_PATH)/bin/trunk" ]; \
	then \
		cargo install trunk --root $(BUILD_PATH); \
	fi
	
build: trunk
	$(BUILD_PATH)/bin/trunk build --release --dist $(PATH_DIST)

setup-minify:
	@if [ ! -f "$(BUILD_PATH)/bin/minifier" ]; \
	then \
		git clone https://github.com/wilsonzlin/minify-html $(BUILD_PATH)/minify-html; \
		chmod a+x $(BUILD_PATH)/minify-html/prebuild.sh; \
		cd $(BUILD_PATH)/minify-html && ./prebuild.sh; \
		cd cli && cargo build --release; \
		mv ./target/release/minify-html-cli $(MINIFIER); \
	fi

# Minify all html, css and js files.
minify: setup-minify build
	@for file in $$(find $(PATH_DIST) -regex '.*\.\(html\|css\|js\)$$'); \
	do \
		FILE_SIZE_SRC=$$(wc -c < $$file); \
		$(MINIFIER) $(MINIFIER_ARGS) --output $$file $$file; \
		FILE_SIZE=$$(wc -c < $$file); \
		echo "Minimized \"$$file\" $$FILE_SIZE_SRC -> $$FILE_SIZE"; \
	done

clean:
	rm -rf $(BUILD_PATH)
