package mattermost

import (
	"flobot/pkg/instance"

	"github.com/mattermost/mattermost-server/model"
)

type Channel struct {
	id     string
	client *model.Client4
	i      instance.Instance
}

func (c *Channel) PostOrReply(to model.Post, msg string) (*model.Post, error) {
	to.ChannelId = c.id
	if to.RootId != "" {
		return Reply(c.i, c.client, to, msg)
	} else {
		return c.Post(model.Post{Message: msg})
	}
}

func (c *Channel) Reply(to model.Post, msg string) (*model.Post, error) {
	to.ChannelId = c.id
	return Reply(c.i, c.client, to, msg)
}

func (c *Channel) Post(post model.Post) (*model.Post, error) {
	post.ChannelId = c.id
	return Post(c.i, c.client, post)
}

func (c *Channel) Channel() (*model.Channel, error) {
	channel, resp := c.client.GetChannel(c.id, "")
	return channel, ToError(resp)
}

func (c *Channel) Space() (string, error) {
	if channel, err := c.Channel(); err != nil {
		return "", err
	} else {
		return channel.TeamId, nil
	}
}
