package instance

import (
	"flobot/pkg/conf"
	"flobot/pkg/store"
	"fmt"
	"log"
	"sync"

	"github.com/pkg/errors"

	"github.com/mattermost/mattermost-server/model"
)

type mattermost struct {
	cfg            conf.Instance
	client         *model.Client4
	team           *model.Team
	ws             *model.WebSocketClient
	me             model.User
	addHandlerLock sync.RWMutex
	handlers       []Handler
	middlewares    []Middleware
	store          store.Store
	running        bool
}

func (i *mattermost) Store() store.Store {
	return i.store
}

func (h Handler) Name() string {
	return fmt.Sprintf("%v", h)
}

func (i *mattermost) Run() error {
	i.running = true
	for {
		select {
		case resp := <-i.ws.EventChannel:
			go i.Handle(resp)
		}
	}
}

func (i *mattermost) AddMiddleware(middleware Middleware) Instance {
	if i.running {
		panic("programming error: cannot add middleware while running")
	}
	i.middlewares = append(i.middlewares, middleware)
	return i
}

func (i *mattermost) AddHandler(handler Handler) Instance {
	if i.running {
		panic("programming error: cannot add handler while running")
	}
	i.handlers = append(i.handlers, handler)
	return i
}

func (i *mattermost) Handle(event *model.WebSocketEvent) {
	for im, middleware := range i.middlewares {
		if cont, err := middleware(i, event); err != nil {
			log.Printf("error from middleware: %d: %v", im, err)
			return
		} else if !cont {
			return
		}
	}

	for ih, handler := range i.handlers {
		if err := handler(i, event); err != nil {
			log.Printf("error from handler: %d: %v", ih, err)
		}
	}
}

func (i *mattermost) WS() *model.WebSocketClient {
	return i.ws
}

func (i *mattermost) Client() *model.Client4 {
	return i.client
}

func (i *mattermost) Cfg() conf.Instance {
	return i.cfg
}

func (i *mattermost) Me() model.User {
	return i.me
}

// NewMattermost creates a new instance and calls internal init before returning.
// If init cannot proceed, it will panic.
func NewMattermost(cfg conf.Instance, store store.Store) Instance {
	i := &mattermost{
		cfg:   cfg,
		store: store,
	}
	i.init()
	return i
}

func (i *mattermost) init() {
	i.initClient()
	i.fetchMe()
	i.websocket()
	i.announce()
}

func (i *mattermost) initClient() {
	i.client = model.NewAPIv4Client(i.cfg.APIv4URL)
	i.client.SetToken(i.cfg.Token)
	log.Println(i.client.ApiUrl)
}

func (i *mattermost) fetchMe() {
	me, resp := i.Client().GetMe("")
	if resp.Error != nil {
		panic(resp.Error.ToJson())
	}
	i.me = *me
}

func (i *mattermost) websocket() {
	if ws, err := model.NewWebSocketClient4(i.cfg.WS, i.cfg.Token); err != nil {
		panic(err.Error())
	} else {
		i.ws = ws
	}
	i.ws.Listen()
}

func (i *mattermost) announce() {
	post := model.Post{}
	post.ChannelId = i.cfg.DebugChan
	post.Message = fmt.Sprintf("bot %s is up", i.cfg.Name)

	post.RootId = ""

	if _, err := i.client.CreatePost(&post); err.Error != nil {
		panic(err.Error.ToJson())
	}
}

func (i *mattermost) SpaceOf(channel string) (string, error) {
	if ichan, err := i.client.GetChannel(channel, ""); err.Error != nil {
		return "", errors.New(err.Error.ToJson())
	} else {
		return ichan.TeamId, nil
	}
}
