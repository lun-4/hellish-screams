run:
	env RUST_LOG=trace cargo run

prod:
	env RUST_LOG=info cargo run --release
