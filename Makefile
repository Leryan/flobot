.PHONY: build
build:
	cargo build --release

.PHONY: vet
vet:
	cargo check
	cargo fmt

.PHONY: test
test:
	cargo test
	cargo fmt

.PHONY: run
run:
	cargo run
