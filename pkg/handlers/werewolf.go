package handlers

import (
	"encoding/json"
	"flobot/pkg/instance"
	werewolf2 "flobot/pkg/werewolf"
	"strings"

	"github.com/pkg/errors"

	"github.com/mattermost/mattermost-server/model"
)

type werewolf struct {
	games map[string]*werewolf2.Game
}

func NewWerewolf() *werewolf {
	return &werewolf{
		games: make(map[string]*werewolf2.Game),
	}
}

func (w *werewolf) handleCommand(channelid, cmd string, i instance.Instance, post model.Post) error {
	if cmd == "start" {
		_, exists := w.games[channelid]
		if !exists {
			w.games[channelid] = &werewolf2.Game{}
			i.Client().CreatePost(&model.Post{
				ChannelId: channelid,
				Message:   "jeu démarré sur ce chan !",
			})
		} else {
			i.Client().CreatePost(&model.Post{
				ChannelId: channelid,
				Message:   "un jeu est déjà en cours sur ce chan",
			})
		}
	} else if cmd == "stop" {
		delete(w.games, channelid)
		i.Client().CreatePost(&model.Post{
			ChannelId: channelid,
			Message:   "jeu arrêté",
		})
	} else {
		i.Client().CreatePost(&model.Post{ChannelId: channelid, Message: "je ne connais pas cette commande", RootId: post.Id})
	}
	return nil
}

func (w *werewolf) Handle(i instance.Instance, event *model.WebSocketEvent) error {
	if event.EventType() == "posted" {
		var post *model.Post
		if err := json.Unmarshal([]byte(event.Data["post"].(string)), &post); err != nil {
			return err
		}

		if strings.HasPrefix(post.Message, "!ww ") {
			return w.handleCommand(post.ChannelId, post.Message[4:], i, *post)
		}
	}
	return errors.New("not implemented")
}
