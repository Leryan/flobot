package handlers

import (
	"encoding/json"
	"flobot/pkg/instance"
	"strings"

	"github.com/mattermost/mattermost-server/model"
	"github.com/pkg/errors"
)

func Avis(i instance.Instance, event *model.WebSocketEvent) error {
	if event.EventType() != "posted" {
		return nil
	}
	var post model.Post
	err := json.Unmarshal([]byte(event.Data["post"].(string)), &post)
	if err != nil {
		return errors.Wrap(err, "avis json decode")
	}

	if strings.Contains(post.Message, " avis ") || strings.HasSuffix(post.Message, " avis") {
		i.Client().CreatePost(&model.Post{
			Message:   "heu si j’peux’m’permettre de donner mon avis, heu ben on te l’a pas demandé à toi d’abord :troll:",
			ChannelId: post.ChannelId,
			RootId:    post.Id,
		})
	}

	return nil
}
