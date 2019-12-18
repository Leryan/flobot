package middlewares

import (
	"flobot/pkg/helpers"
	"flobot/pkg/instance"
	"regexp"

	"github.com/mattermost/mattermost-server/model"
)

var emmerde = regexp.MustCompile(".*flop.+[aà].+quit[t]?[eé]?.+([a-zA-Z]+).+c.+")

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

	if emmerde.MatchString(post.Message) {
		return false, helpers.Reply(i, *post, "Me prends pas pour un dindon toi !")
	}

	return true, nil
}
