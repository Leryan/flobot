.PHONY: build
build:
	cargo build --release

.PHONY: vet
vet:
	cargo check
	cargo fmt
