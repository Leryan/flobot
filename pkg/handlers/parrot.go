package handlers

import (
	"encoding/json"
	"flobot/pkg/instance"
	"strings"

	"github.com/pkg/errors"

	"github.com/mattermost/mattermost-server/model"
)

func Parrot(i instance.Instance, event model.WebSocketEvent) error {
	if event.EventType() != "posted" {
		return nil
	}

	var post model.Post
	err := json.Unmarshal([]byte(event.Data["post"].(string)), &post)
	if err != nil {
		return errors.Wrap(err, "parrot json decode")
	}

	pref := "!perroquet "
	if strings.HasPrefix(post.Message, pref) {
		cmd := post.Message[len(pref):]

		_, err := i.Client().Chan.Get(post.ChannelId).Post(model.Post{
			ChannelId: post.ChannelId,
			Message:   cmd,
		})

		return err
	}
	return nil
}
