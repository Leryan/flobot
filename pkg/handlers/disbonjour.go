package handlers

import (
	"encoding/json"
	"strings"

	"flobot/pkg/instance"
	"github.com/mattermost/mattermost-server/model"
)

func DisBonjour(i *instance.Instance, event *model.WebSocketEvent) error {
	if event.EventType() != "posted" {
		return nil
	}

	var post model.Post
	if err := json.Unmarshal([]byte(event.Data["post"].(string)), &post); err != nil {
		return err
	}

	if strings.HasPrefix(post.Message, "!disbonjour") {
		i.Client().CreatePost(&model.Post{
			Message:   "Bonjour :wave: ! Moi c’est FloBot. Je sais pas faire grand chose, mais le grand @flop y travaille dur dur dur :3\n\nEssaye `!perroquet coucou !`",
			ChannelId: post.ChannelId,
		})
	}

	return nil
}
