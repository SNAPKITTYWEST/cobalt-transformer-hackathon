.PHONY: all build test clean run-example

all: build test

build:
	cd cobol-transformer && cargo build --release

test:
	cd cobol-transformer && cargo test

clean:
	cd cobol-transformer && cargo clean

run-example:
	cd cobol-transformer && cargo run -- tests/fixtures/hello.cob

install:
	cd cobol-transformer && cargo install --path .

check:
	cd cobol-transformer && cargo check

fmt:
	cd cobol-transformer && cargo fmt

clippy:
	cd cobol-transformer && cargo clippy

# Made with Bob
