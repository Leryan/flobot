package mattermost

import (
	"flobot/pkg/instance"

	"github.com/mattermost/mattermost-server/model"
)

func NewClient(i instance.Instance, client *model.Client4) instance.Client {
	return instance.Client{
		Me:   &Me{client: client},
		Chan: &Chan{client: client, i: i},
	}
}
