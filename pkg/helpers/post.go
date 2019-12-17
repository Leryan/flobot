package helpers

import (
	"encoding/json"

	"github.com/mattermost/mattermost-server/model"
)

func DecodePost(event *model.WebSocketEvent) (*model.Post, error) {
	if event.EventType() != "posted" {
		return nil, nil
	}

	var post model.Post
	if err := json.Unmarshal([]byte(event.Data["post"].(string)), &post); err != nil {
		return nil, err
	}

	return &post, nil
}
