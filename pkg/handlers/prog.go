package handlers

import (
	"flobot/pkg/helpers"
	"flobot/pkg/instance"
	"flobot/pkg/instance/mattermost"
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
	Name         string        `json:"Name"`
	Owner        string        `json:"Owner"`
	Instructions []instruction `json:"Instructions"`
	vars         map[string]string
	err          error
}

func (p *prog) Run() string {
	p.vars = make(map[string]string)
	p.err = nil
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

func (p *proghandler) Handle(i instance.Instance, event model.WebSocketEvent) error {
	post, err := mattermost.DecodePost(event)
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

	c := i.Client().Chan.Get(post.ChannelId)

	if len(cmd) < 1 {
		_, err := c.Reply(*post, "au moins 1 param")
		return err
	}

	defer func() {
		if r := recover(); r != nil {
			c.Reply(*post, fmt.Sprintf("cétoupouri: %v", r))
		}
	}()

	if cmd[0] == "create" {
		p.progs.LoadOrStore(post.UserId+cmd[1], newProg(cmd[1], post.UserId))
		return helpers.Discard(c.Reply(*post, "programme créé rien que pour toi :3"))
	} else if cmd[0] == "load" {
		progsProg, ok := p.progs.Load(post.UserId + cmd[1])
		if ok {
			p.current.Store(post.UserId, progsProg)
			return helpers.Discard(c.Reply(*post, "chargé !"))
		} else {
			var tmp prog
			if err := p.store.Collection("prog").Get(post.UserId, &tmp); err != nil {
				return helpers.Discard(c.Reply(*post, err.Error()))
			}
			if tmp.Name == "" {
				return helpers.Discard(c.Reply(*post, "l'existe pô"))
			}

			p.progs.Store(post.UserId+cmd[1], &tmp)
			p.current.Store(post.UserId, &tmp)
			return helpers.Discard(c.Reply(*post, "chargé depuis la db !"))
		}
	} else if cmd[0] == "del" {
		p.progs.Delete(post.UserId + cmd[1])
		p.current.Delete(post.UserId)
		return helpers.Discard(c.Reply(*post, "apu !"))
	}

	cp, ok := p.current.Load(post.UserId)
	if !ok {
		return helpers.Discard(c.Reply(*post, "faut d’abord charger un prog"))
	}

	P := cp.(*prog)

	if cmd[0] == "save" {
		p.store.Collection("prog").Set(post.UserId, P)
		return helpers.Discard(c.Reply(*post, "sauvéééé"))
	} else if cmd[0] == "a" {
		if len(P.Instructions) >= 200 {
			return helpers.Discard(c.Reply(*post, "limite d’instructions atteinte"))
		}
		P.Instructions = append(P.Instructions, instruction{
			Op:     cmd[1],
			Params: cmd[2:],
		})
		return helpers.Discard(c.Reply(*post, "remplacée"))
	} else if cmd[0] == "r" {
		idx, err := strconv.ParseUint(cmd[1], 10, 64)
		if err != nil {
			return helpers.Discard(c.Reply(*post, err.Error()))
		}
		P.Instructions[idx] = instruction{Op: cmd[2], Params: cmd[3:]}
		return helpers.Discard(c.Reply(*post, "remplacé"))
	} else if cmd[0] == "i" {
		if len(P.Instructions) >= 200 {
			return helpers.Discard(c.Reply(*post, "limite d’instructions atteinte"))
		}
		return helpers.Discard(c.Reply(*post, "pas implementé"))
	} else if cmd[0] == "run" {
		return helpers.Discard(c.Reply(*post, P.Run()))
	}
	return helpers.Discard(c.Reply(*post, "nan ça existe pô ça"))
}
