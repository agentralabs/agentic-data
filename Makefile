.PHONY: build build-debug test test-unit lint lint-fmt lint-clippy bench clean install check

build:
	cargo build --release -j 1

build-debug:
	cargo build -j 1

test:
	cargo test --all -j 1 -- --test-threads=1

test-unit:
	cargo test -p agentic-data -j 1 -- --test-threads=1

lint: lint-fmt lint-clippy

lint-fmt:
	cargo fmt --check

lint-clippy:
	cargo clippy --all -- -D warnings

bench:
	cargo bench -j 1

clean:
	cargo clean

install:
	cargo install --path crates/agentic-data-cli -j 1
	cargo install --path crates/agentic-data-mcp -j 1

check:
	bash scripts/check-canonical-sister.sh
	bash scripts/check-install-commands.sh
