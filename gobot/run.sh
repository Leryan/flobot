#!/bin/sh

export BOT_NAME="dev"
export BOT_DEBUG_CHAN="hokmrw93c7ghfghhjef1zg8h9h"
export BOT_APIV4URL="http://localhost:8065"
export BOT_TOKEN="nqphehwbkpfe5fcr48k5rq4zuo"
export BOT_WS="ws://localhost:8065"
export BOT_TEAM_NAME="First"

go run cmd/bot/main.go
