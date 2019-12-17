package handlers

import (
	"encoding/json"
	"log"
	"regexp"
	"strings"

	"flobot/pkg/instance"

	"github.com/mattermost/mattermost-server/model"
)

func EmmerdeMaison(i instance.Instance, event *model.WebSocketEvent) error {
	if event.EventType() != "posted" {
		return nil
	}

	var post model.Post
	if err := json.Unmarshal([]byte(event.Data["post"].(string)), &post); err != nil {
		return err
	}

	auto := false
	answerTo := ""
	r := regexp.MustCompile(".*@flop [aà] quit[t]?[eé] le.*")
	if matched := r.MatchString(strings.ToLower(post.Message)); matched {
		auto = true
		answerTo = post.Id
	}

	cmd := strings.HasPrefix(post.Message, "!emmerde")

	log.Printf("cmd: %v | auto: %v", cmd, auto)

	if cmd || auto {
		i.Client().CreatePost(&model.Post{
			Message:   "Y dit qu’y vous emmerde et qu’y rentre à sa maison ! :cartman:",
			ChannelId: post.ChannelId,
			RootId:    answerTo,
		})
	}

	return nil
}
