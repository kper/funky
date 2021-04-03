cargo build --verbose --all && cargo test --verbose --all && cargo clippy --all-features -- -D warnings && cd ide && cargo build && cargo test --verbose
