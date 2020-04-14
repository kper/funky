all: test clippy audit

test:
	cargo test

audit:
	cargo audit -D

clippy:
	cargo clippy --all-features -- -D warnings

setup:
	cargo install cargo-audit
	rustup component add clippy
