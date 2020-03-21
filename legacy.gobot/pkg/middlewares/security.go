package middlewares

import (
	"flobot/pkg/instance"
	"flobot/pkg/instance/mattermost"
	"regexp"
	"strings"

	"github.com/mattermost/mattermost-server/model"
)

var emmerde = regexp.MustCompile(".*flop.+quit.+")
var null = " "

func Security(i instance.Instance, event *model.WebSocketEvent) (bool, error) {
	post, err := mattermost.DecodePost(*event)
	if err != nil {
		return false, err
	}
	if post == nil {
		return true, nil
	}

	me, err := i.Client().Me.Me()
	if err != nil {
		return false, err
	}

	if post.UserId == me.Id {
		return false, nil
	}

	if strings.Contains(post.Message, null) {
		return false, nil
	}

	return true, nil
}
