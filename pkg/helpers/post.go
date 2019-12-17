package helpers

import (
	"encoding/json"
	"flobot/pkg/instance"

	"github.com/pkg/errors"

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

func Reply(i instance.Instance, post model.Post, msg string) error {
	_, err := i.Client().CreatePost(&model.Post{Message: msg, RootId: post.Id, ChannelId: post.ChannelId})
	if err.Error != nil {
		return errors.New(err.Error.DetailedError)
	}
	return nil
}
