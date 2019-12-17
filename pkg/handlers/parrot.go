package handlers

import (
	"encoding/json"
	"flobot/pkg/instance"
	"strings"

	"github.com/pkg/errors"

	"github.com/mattermost/mattermost-server/model"
)

func Parrot(i instance.Instance, event *model.WebSocketEvent) error {
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

		if strings.Contains(cmd, "!perroquet") {
			i.Client().CreatePost(&model.Post{
				Message:   "me prends pas pour un dindon toi !",
				ChannelId: post.ChannelId,
				RootId:    post.Id,
			})
			return nil
		}

		_, resp := i.Client().CreatePost(&model.Post{
			ChannelId: post.ChannelId,
			Message:   cmd,
		})

		if resp.Error != nil {
			return errors.New(resp.Error.DetailedError)
		}
	}
	return nil
}
