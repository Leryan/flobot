package handlers

import (
	"encoding/json"
	"flobot/pkg/instance"
	"fmt"

	"github.com/pkg/errors"

	"github.com/mattermost/mattermost-server/model"
)

func Parrot(i *instance.Instance, event *model.WebSocketEvent) error {
	if event.EventType() != "posted" {
		return nil
	}

	var post model.Post
	err := json.Unmarshal([]byte(event.Data["post"].(string)), &post)
	if err != nil {
		return errors.Wrap(err, "parrot json decode")
	}

	_, resp := i.Client().CreatePost(&model.Post{
		ChannelId: i.Cfg().DebugChan,
		Message:   fmt.Sprintf("Parrrrrrrrrot: %s", post.Message),
	})

	if resp.Error != nil {
		return errors.New(resp.Error.DetailedError)
	}

	return nil
}
