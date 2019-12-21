package handlers

import (
	"encoding/json"
	"flobot/pkg/instance"
	"log"

	"github.com/davecgh/go-spew/spew"

	"github.com/mattermost/mattermost-server/model"
)

func Console(i instance.Instance, event model.WebSocketEvent) error {
	out := make(map[string]interface{})

	if err := json.Unmarshal([]byte(event.ToJson()), &out); err != nil {
		return err
	}

	log.Printf("%s", spew.Sdump(out))
	return nil
}
