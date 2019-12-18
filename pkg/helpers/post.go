package helpers

import (
	"encoding/json"
	"flobot/pkg/instance"
	"fmt"

	"github.com/mattermost/mattermost-server/model"
)

type Error struct {
	Post model.Post
	Err  string
}

func (e Error) Error() string {
	return e.Err
}

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
	rootid := post.RootId
	if post.RootId == "" {
		rootid = post.Id
	}
	return Post(i, model.Post{
		Message:   msg,
		RootId:    rootid,
		ChannelId: post.ChannelId,
		ParentId:  post.ParentId,
	})
}

func Post(i instance.Instance, post model.Post) error {
	_, err := i.Client().CreatePost(&post)
	if err.Error != nil {
		i.Client().CreatePost(&model.Post{
			ChannelId: i.Cfg().DebugChan,
			Message:   fmt.Sprintf("bug: `%s` from message: \n```\n%s\n```", err.Error.ToJson(), post.ToJson()),
		})
		return Error{
			Post: post,
			Err:  err.Error.ToJson(),
		}
	}
	return nil
}
