package handlers

import (
	"flobot/pkg/helpers"
	"flobot/pkg/instance"
	"flobot/pkg/store"
	"fmt"
	"strconv"
	"strings"
	"sync"

	"github.com/mattermost/mattermost-server/model"
)

func NewProg() *proghandler {
	return &proghandler{
		store: store.NewJSONStore("botdb"),
	}
}

type instruction struct {
	Op     string   `json:"op"`
	Params []string `json:"params"`
}

type prog struct {
	Name         string            `json:"Name"`
	Owner        string            `json:"Owner"`
	Instructions []instruction     `json:"Instructions"`
	vars         map[string]string `json:"-"`
	err          error             `json:"-"`
}

func (p *prog) Run() string {
	msg := ""
	for _, i := range p.Instructions {
		msg = fmt.Sprintf("%s * %s -> %v\n", msg, i.Op, i.Params)
	}
	return msg + "\n\nmais c’est pas encore implémenté"
}

func newProg(name, owner string) *prog {
	return &prog{
		Name:         name,
		Owner:        owner,
		Instructions: make([]instruction, 0),
		vars:         make(map[string]string),
		err:          nil,
	}
}

type proghandler struct {
	progs   sync.Map
	current sync.Map
	store   store.Store
}

func (p *proghandler) Handle(i instance.Instance, event *model.WebSocketEvent) error {
	post, err := helpers.DecodePost(event)
	if err != nil {
		return err
	}
	if post == nil {
		return nil
	}

	if !strings.HasPrefix(post.Message, "!prog ") {
		return nil
	}

	cmd := strings.Split(post.Message[6:], " ")

	if len(cmd) < 1 {
		i.Client().CreatePost(&model.Post{Message: "au moins 1 param", RootId: post.Id, ChannelId: post.ChannelId})
		return nil
	}

	defer func() {
		if r := recover(); r != nil {
			i.Client().CreatePost(&model.Post{Message: fmt.Sprintf("cétoupouri: %v", r), RootId: post.Id, ChannelId: post.ChannelId})
		}
	}()

	if cmd[0] == "create" {
		p.progs.LoadOrStore(post.UserId+cmd[1], newProg(cmd[1], post.UserId))
		i.Client().CreatePost(&model.Post{Message: "programme créé rien que pour toi :3", RootId: post.Id, ChannelId: post.ChannelId})
		return nil
	}

	if cmd[0] == "load" {
		progsProg, ok := p.progs.Load(post.UserId + cmd[1])
		if ok {
			p.current.Store(post.UserId, progsProg)
			i.Client().CreatePost(&model.Post{Message: "chargé !", RootId: post.Id, ChannelId: post.ChannelId})
		} else {
			var tmp prog
			if err := p.store.Collection("prog").Get(post.UserId, &tmp); err != nil {
				i.Client().CreatePost(&model.Post{Message: err.Error(), RootId: post.Id, ChannelId: post.ChannelId})
				return nil
			}
			if tmp.Name == "" {
				i.Client().CreatePost(&model.Post{Message: "l'existe pô", RootId: post.Id, ChannelId: post.ChannelId})
				return nil
			}

			p.progs.Store(post.UserId+cmd[1], &tmp)
			p.current.Store(post.UserId, &tmp)
			i.Client().CreatePost(&model.Post{Message: "chargé depuis la db !", RootId: post.Id, ChannelId: post.ChannelId})
		}
		return nil
	}

	if cmd[0] == "del" {
		p.progs.Delete(post.UserId + cmd[1])
		p.current.Delete(post.UserId)
		i.Client().CreatePost(&model.Post{Message: "apu !", RootId: post.Id, ChannelId: post.ChannelId})
		return nil
	}

	cp, ok := p.current.Load(post.UserId)
	if !ok {
		i.Client().CreatePost(&model.Post{Message: "faut d’abord charger un prog", RootId: post.Id, ChannelId: post.ChannelId})
		return nil
	}

	P := cp.(*prog)

	if cmd[0] == "save" {
		p.store.Collection("prog").Set(post.UserId, P)
		i.Client().CreatePost(&model.Post{Message: "saved", RootId: post.Id, ChannelId: post.ChannelId})
		return nil
	}

	if cmd[0] == "a" {
		if len(P.Instructions) >= 200 {
			i.Client().CreatePost(&model.Post{Message: "limite d’instruction atteinte", RootId: post.Id, ChannelId: post.ChannelId})
			return nil
		}
		P.Instructions = append(P.Instructions, instruction{
			Op:     cmd[1],
			Params: cmd[2:],
		})
		i.Client().CreatePost(&model.Post{Message: "added", RootId: post.Id, ChannelId: post.ChannelId})
	}

	if cmd[0] == "r" {
		idx, err := strconv.ParseUint(cmd[1], 10, 64)
		if err != nil {
			i.Client().CreatePost(&model.Post{Message: err.Error(), RootId: post.Id, ChannelId: post.ChannelId})
			return nil
		}
		P.Instructions[idx] = instruction{Op: cmd[2], Params: cmd[3:]}
		i.Client().CreatePost(&model.Post{Message: "replaced", RootId: post.Id, ChannelId: post.ChannelId})
	}

	if cmd[0] == "i" {
		if len(P.Instructions) >= 200 {
			i.Client().CreatePost(&model.Post{Message: "limite d’instruction atteinte", RootId: post.Id, ChannelId: post.ChannelId})
			return nil
		}
		i.Client().CreatePost(&model.Post{Message: "not implemented", RootId: post.Id, ChannelId: post.ChannelId})
	}

	if cmd[0] == "run" {
		i.Client().CreatePost(&model.Post{Message: P.Run(), RootId: post.Id, ChannelId: post.ChannelId})
		return nil
	}

	return nil
}
