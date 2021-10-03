.PHONY: build
build:
	cargo build --release
	strip target/release/flobot

.PHONY: vet
vet:
	cargo check

.PHONY: test
test: vet
	cargo test

.PHONY: run
run:
	RUST_BACKTRACE=1 cargo run

.PHONY: deploy
deploy: test build
	scp target/release/flobot srv.leila:/home/bot/flobot.upgrade
	ssh srv.leila systemctl restart bot