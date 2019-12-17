package middlewares

import (
	"flobot/pkg/helpers"
	"flobot/pkg/instance"

	"github.com/mattermost/mattermost-server/model"
)

func Security(i instance.Instance, event *model.WebSocketEvent) (bool, error) {
	post, err := helpers.DecodePost(event)
	if err != nil {
		return false, err
	}
	if post == nil {
		return true, nil
	}

	if post.UserId == i.Me().Id {
		return false, nil
	}

	return true, nil
}
