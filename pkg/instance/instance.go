package instance

import (
	"flobot/pkg/conf"
	"fmt"
	"log"
	"sync"

	"github.com/mattermost/mattermost-server/model"
)

type Instance struct {
	cfg            conf.Instance
	client         *model.Client4
	team           *model.Team
	ws             *model.WebSocketClient
	addHandlerLock sync.RWMutex
	handlers       []Handler
}

type Handler func(i *Instance, event *model.WebSocketEvent) error

func (h Handler) Name() string {
	return fmt.Sprintf("%v", h)
}

func (i *Instance) Run() error {
	for {
		select {
		case resp := <-i.ws.EventChannel:
			i.Handle(resp)
		}
	}
}

func (i *Instance) AddHandler(handler Handler) {
	i.addHandlerLock.Lock()
	defer i.addHandlerLock.Unlock()
	i.handlers = append(i.handlers, handler)
}

func (i *Instance) Handle(event *model.WebSocketEvent) {
	i.addHandlerLock.RLock()
	defer i.addHandlerLock.RUnlock()
	for _, handler := range i.handlers {
		go i.handleError(handler, handler(i, event))
	}
}

func (i *Instance) handleError(h Handler, err error) {
	log.Printf("error from handler %s: %v", h.Name(), err)
}

func (i *Instance) WS() *model.WebSocketClient {
	return i.ws
}

func (i *Instance) Client() *model.Client4 {
	return i.client
}

func (i *Instance) Cfg() conf.Instance {
	return i.cfg
}

// NewFromCfg creates a new instance and calls internal init before returning.
// If init cannot proceed, it will panic.
func NewFromCfg(cfg conf.Instance) *Instance {
	i := &Instance{
		cfg: cfg,
	}
	i.init()
	return i
}

func (i *Instance) init() {
	i.initClient()
	i.websocket()
	i.announce()
}

func (i *Instance) initClient() {
	i.client = model.NewAPIv4Client(i.cfg.APIv4URL)
	i.client.SetToken(i.cfg.Token)
	log.Println(i.client.ApiUrl)
}

func (i *Instance) websocket() {
	if ws, err := model.NewWebSocketClient4(i.cfg.WS, i.cfg.Token); err != nil {
		panic(err.Error())
	} else {
		i.ws = ws
	}
	i.ws.Listen()
}

func (i *Instance) announce() {
	post := &model.Post{}
	post.ChannelId = i.cfg.DebugChan
	post.Message = fmt.Sprintf("bot %s is up", i.cfg.Name)

	post.RootId = ""

	if _, resp := i.client.CreatePost(post); resp.Error != nil {
		panic(resp.Error.Error())
	}
}
