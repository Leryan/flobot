package instance

import (
	"flobot/pkg/conf"
	"flobot/pkg/store"

	"github.com/mattermost/mattermost-server/model"
)

type Instance interface {
	Cfg() conf.Instance
	Client() Client
	Handle(event model.WebSocketEvent)
	AddHandler(handler Handler) Instance
	AddMiddleware(middleware Middleware) Instance
	Run() error
	Store() store.Store
}

type Me interface {
	Me() (*model.User, error)
	Spaces() ([]string, error)
}

type Channel interface {
	Post(post model.Post) (*model.Post, error)
	Reply(to model.Post, msg string) (*model.Post, error)
	PostOrReply(to model.Post, msg string) (*model.Post, error)
	Channel() (*model.Channel, error)
	Space() (string, error)
}

type Chan interface {
	Get(id string) Channel
}

type Client struct {
	Me   Me
	Chan Chan
}

type Handler func(i Instance, event model.WebSocketEvent) error
type Middleware func(i Instance, event *model.WebSocketEvent) (bool, error) // returns true -> continue, false -> stop
