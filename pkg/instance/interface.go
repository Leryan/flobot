package instance

import (
	"flobot/pkg/conf"

	"github.com/mattermost/mattermost-server/model"
)

type Instance interface {
	Me() model.User
	Cfg() conf.Instance
	Client() *model.Client4
	WS() *model.WebSocketClient
	Handle(event *model.WebSocketEvent)
	AddHandler(handler Handler) Instance
	AddMiddleware(middleware Middleware) Instance
	Run() error
}

type Handler func(i Instance, event *model.WebSocketEvent) error
type Middleware func(i Instance, event *model.WebSocketEvent) (bool, error) // returns true -> continue, false -> stop
