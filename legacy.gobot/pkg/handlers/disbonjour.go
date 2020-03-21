package handlers

import (
	"encoding/json"
	"strings"

	"flobot/pkg/instance"

	"github.com/mattermost/mattermost-server/model"
)

func DisBonjour(i instance.Instance, event model.WebSocketEvent) error {
	if event.EventType() != "posted" {
		return nil
	}

	var post model.Post
	if err := json.Unmarshal([]byte(event.Data["post"].(string)), &post); err != nil {
		return err
	}

	if strings.HasPrefix(post.Message, "!disbonjour") {
		_, err := i.Client().Chan.Get(post.ChannelId).Post(
			model.Post{
				Message:   "Bonjour :wave: ! Moi c’est FloBot, je sais pas faire grand chose, mais tu peux essayer `!perroquet coucou` ou `!trigger list` !\n\nhttps://gitlab.com/Leryan/flobot",
				ChannelId: post.ChannelId,
			},
		)
		return err
	}

	return nil
}
