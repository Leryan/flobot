package handlers

import (
	"encoding/json"
	"flobot/pkg/instance"
	"strings"

	"github.com/mattermost/mattermost-server/model"
	"github.com/pkg/errors"
)

func Avis(i instance.Instance, event model.WebSocketEvent) error {
	if event.EventType() != "posted" {
		return nil
	}
	var post model.Post
	err := json.Unmarshal([]byte(event.Data["post"].(string)), &post)
	if err != nil {
		return errors.Wrap(err, "avis json decode")
	}

	if strings.Contains(post.Message, " avis ") || strings.HasSuffix(post.Message, " avis") {
		_, err := i.Client().Chan.Get(post.ChannelId).Reply(
			post,
			"heu si j’peux’m’permettre de donner mon avis, heu ben on te l’a pas demandé à toi d’abord :troll:",
		)
		return err
	}

	return nil
}
