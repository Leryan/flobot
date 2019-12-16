#!/bin/sh

export BOT_NAME="dev"
export BOT_DEBUG_CHAN="b8rgca6m3pdbuge5k5spx6zbor"
export BOT_APIV4URL="http://localhost:8065"
export BOT_TOKEN="ig9a6crj4bnnmy8cwsaxh1ikww"
export BOT_WS="ws://localhost:8065"
export BOT_TEAM_NAME="First"

go run cmd/bot/main.go
