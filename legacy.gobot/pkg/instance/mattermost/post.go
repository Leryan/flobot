package mattermost

import (
	"encoding/json"
	"flobot/pkg/instance"
	"fmt"

	"github.com/mattermost/mattermost-server/model"
)

func DecodePost(event model.WebSocketEvent) (*model.Post, error) {
	if event.EventType() != "posted" {
		return nil, nil
	}

	var post model.Post
	if err := json.Unmarshal([]byte(event.Data["post"].(string)), &post); err != nil {
		return nil, err
	}

	return &post, nil
}

func Reply(i instance.Instance, client *model.Client4, post model.Post, msg string) (*model.Post, error) {
	rootid := post.RootId
	if post.RootId == "" {
		rootid = post.Id
	}
	return Post(i, client, model.Post{
		Message:   msg,
		RootId:    rootid,
		ChannelId: post.ChannelId,
		ParentId:  post.ParentId,
	})
}

func Post(i instance.Instance, client *model.Client4, post model.Post) (*model.Post, error) {
	npost, err := client.CreatePost(&post)
	if err.Error != nil {
		client.CreatePost(&model.Post{
			ChannelId: i.Cfg().DebugChan,
			Message:   fmt.Sprintf("bug: `%s` from message: \n```\n%s\n```", err.Error.ToJson(), post.ToJson()),
		})
		return nil, instance.Error{
			Code:   err.StatusCode,
			Status: err.Error.ToJson(),
		}
	}
	return npost, nil
}
