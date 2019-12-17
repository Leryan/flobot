.PHONY: vet
vet:
	go vet ./...
	go mod tidy

.PHONY: build_debug
build_debug:
	go build ./cmd/bot/

.PHONY: build
build: build_debug
	strip bot

.PHONY: build
deploy: build
	scp bot srv.leila:/home/bot/bot-upgrade
	ssh srv.leila systemctl stop bot
	ssh srv.leila mv /home/bot/bot-upgrade /home/bot/bot
	ssh srv.leila systemctl start bot
