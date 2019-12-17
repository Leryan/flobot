package handlers

import (
	"fmt"
	"regexp"
	"strings"
	"sync"

	"flobot/pkg/helpers"
	"flobot/pkg/instance"

	"github.com/mattermost/mattermost-server/model"
)

var triggerAdd = regexp.MustCompile("!trigger add ([a-zA-Z0-9_]+) (.+)")
var triggerDel = regexp.MustCompile("!trigger del ([a-zA-Z0-9_]+)")

type triggered struct {
	triggers sync.Map
}

func (t *triggered) handleTriggerAdd(i instance.Instance, post *model.Post) error {
	subs := triggerAdd.FindStringSubmatch(post.Message)

	if len(subs) != 3 {
		i.Client().CreatePost(&model.Post{Message: "je comprends pô", RootId: post.Id, ChannelId: post.ChannelId})
		return nil
	}

	t.triggers.Store(strings.ToLower(subs[1]), subs[2])

	return nil
}

func (t *triggered) handleTriggerDel(i instance.Instance, post *model.Post) error {
	subs := triggerDel.FindStringSubmatch(post.Message)

	if len(subs) != 2 {
		i.Client().CreatePost(&model.Post{Message: "t’y es preeeeesque", RootId: post.Id, ChannelId: post.ChannelId})
		return nil
	}
	t.triggers.Delete(subs[1])
	i.Client().CreatePost(&model.Post{Message: "Caroline… supprimée.", RootId: post.Id, ChannelId: post.ChannelId})
	return nil
}

func (t *triggered) handleTriggerList(i instance.Instance, post *model.Post) error {
	trigs := make([]string, 0)
	t.triggers.Range(func(key interface{}, value interface{}) bool {
		trigs = append(trigs, fmt.Sprintf(" * `%s`: %s", key, value))
		return true
	})

	msg := "Ah, ben yen a pas.\n\n * `!trigger add <nom> <ce que tu veux>`\n * `!trigger del <nom>`\n * `!trigger list`\n"
	if len(trigs) > 0 {
		msg = "Liste des triggers :triggered:\n\n"
		msg = msg + strings.Join(trigs, "\n")
	}

	i.Client().CreatePost(&model.Post{Message: msg, ChannelId: post.ChannelId})

	return nil
}

func (t *triggered) handleMessage(i instance.Instance, post *model.Post) error {
	t.triggers.Range(func(key interface{}, value interface{}) bool {
		if strings.Contains(strings.ToLower(post.Message), fmt.Sprintf("*%s*", key)) {
			i.Client().CreatePost(&model.Post{Message: fmt.Sprintf("%s", value), ChannelId: post.ChannelId})
			return false
		}
		return true
	})
	return nil
}

func (t *triggered) Handler(i instance.Instance, event *model.WebSocketEvent) error {
	if post, err := helpers.DecodePost(event); err != nil {
		return nil
	} else if post != nil {
		if strings.HasPrefix(post.Message, "!trigger add ") {
			return t.handleTriggerAdd(i, post)
		} else if strings.HasPrefix(post.Message, "!trigger del") {
			return t.handleTriggerDel(i, post)
		} else if strings.HasPrefix(post.Message, "!trigger list") {
			return t.handleTriggerList(i, post)
		} else {
			return t.handleMessage(i, post)
		}
	}
	return nil
}

func NewTriggered(dbPath string) instance.Handler {
	t := &triggered{}
	return t.Handler
}
