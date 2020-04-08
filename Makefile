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
	RUST_BACKTRACE=1 cargo run

.PHONY: deploy
deploy:
	systemctl stop bot
	cp target/release/flobot /home/bot/
	rsync -avKSHc --delete ./migrations/ /home/bot/migrations/
	systemctl start bot
