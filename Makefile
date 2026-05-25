# abc-scaffold Makefile
# Default targets assume Rust. Replace with your project's build commands.

.PHONY: build release test fmt lint clean hooks

build:
	cargo build

release:
	cargo build --release

test:
	cargo test

fmt:
	cargo fmt

lint:
	cargo clippy

clean:
	cargo clean

hooks:
	ln -sf "$(PWD)/hooks/pre-push" .git/hooks/pre-push
