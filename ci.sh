cargo build --verbose --all && cargo test --verbose --all && cargo clippy --all-features -- -D warnings && cd ifds && cargo build && cargo test --verbose
