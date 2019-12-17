package handlers

import (
	"encoding/json"
	"strings"

	"flobot/pkg/instance"

	"github.com/mattermost/mattermost-server/model"
)

func HyperCon(i instance.Instance, event *model.WebSocketEvent) error {
	if event.EventType() != "posted" {
		return nil
	}

	var post model.Post
	if err := json.Unmarshal([]byte(event.Data["post"].(string)), &post); err != nil {
		return err
	}

	if strings.Contains(post.Message, "hyper con") {
		i.Client().CreatePost(&model.Post{Message: ":perceval:", ChannelId: post.ChannelId, RootId: post.Id})
	}

	return nil
}
