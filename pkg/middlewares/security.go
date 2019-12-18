package middlewares

import (
	"flobot/pkg/instance"
	"flobot/pkg/instance/mattermost"
	"regexp"
	"strings"

	"github.com/mattermost/mattermost-server/model"
)

var emmerde = regexp.MustCompile(".*flop.+quit.+")

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

	msg := strings.ToLower(post.Message)
	if strings.Contains(msg, "flop") || strings.Contains(msg, "quit") {
		_, err := i.Client().Chan.Get(post.ChannelId).Reply(*post, "Me prends pas pour un dindon toi !")
		return false, err
	}

	return true, nil
}
