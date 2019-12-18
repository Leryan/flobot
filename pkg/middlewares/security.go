package middlewares

import (
	"flobot/pkg/helpers"
	"flobot/pkg/instance"
	"regexp"
	"strings"

	"github.com/mattermost/mattermost-server/model"
)

var emmerde = regexp.MustCompile(".*flop.+quit.+")

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

	msg := strings.ToLower(post.Message)
	if strings.Contains(msg, "flop") || strings.Contains(msg, "quit") {
		return false, helpers.Reply(i, *post, "Me prends pas pour un dindon toi !")
	}

	return true, nil
}
