package mattermost

import (
	"flobot/pkg/instance"

	"github.com/mattermost/mattermost-server/model"
)

type Chan struct {
	client *model.Client4
	i      instance.Instance
}

func (c *Chan) Get(id string) instance.Channel {
	return &Channel{id: id, client: c.client, i: c.i}
}
