package handlers

import (
	"flobot/pkg/instance"
	"log"

	"github.com/mattermost/mattermost-server/model"
)

func Console(i *instance.Instance, event *model.WebSocketEvent) error {
	log.Printf("%s", event.ToJson())
	return nil
}
