package instance

import (
	"flobot/pkg/conf"
	"fmt"
	"log"
	"sync"

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
}

func (h Handler) Name() string {
	return fmt.Sprintf("%v", h)
}

func (i *mattermost) Run() error {
	for {
		select {
		case resp := <-i.ws.EventChannel:
			i.Handle(resp)
		}
	}
}

func (i *mattermost) AddMiddleware(middleware Middleware) Instance {
	i.addHandlerLock.Lock()
	defer i.addHandlerLock.Unlock()
	i.middlewares = append(i.middlewares, middleware)
	return i
}

func (i *mattermost) AddHandler(handler Handler) Instance {
	i.addHandlerLock.Lock()
	defer i.addHandlerLock.Unlock()
	i.handlers = append(i.handlers, handler)
	return i
}

func (i *mattermost) Handle(event *model.WebSocketEvent) {
	i.addHandlerLock.RLock()
	defer i.addHandlerLock.RUnlock()
	for _, middleware := range i.middlewares {
		cont, err := middleware(i, event)
		if err != nil {
			log.Printf("error from middleware: %v", err)
			return
		}
		if !cont {
			return
		}
	}
	for _, handler := range i.handlers {
		go i.handleError(handler, handler(i, event))
	}
}

func (i *mattermost) handleError(h Handler, err error) {
	log.Printf("error from handler %s: %v", h.Name(), err)
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
func NewMattermost(cfg conf.Instance) Instance {
	i := &mattermost{
		cfg: cfg,
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
		panic(resp.Error.DetailedError)
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
	post := &model.Post{}
	post.ChannelId = i.cfg.DebugChan
	post.Message = fmt.Sprintf("bot %s is up", i.cfg.Name)

	post.RootId = ""

	if _, resp := i.client.CreatePost(post); resp.Error != nil {
		panic(resp.Error.Error())
	}
}
