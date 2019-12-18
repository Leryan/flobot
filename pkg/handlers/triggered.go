package handlers

import (
	"fmt"
	"regexp"
	"strings"
	"sync"

	"flobot/pkg/helpers"
	"flobot/pkg/instance"
	"flobot/pkg/store"

	"github.com/mattermost/mattermost-server/model"
)

var triggerAdd = regexp.MustCompile("!trigger add ([a-zA-Z0-9_]+) (.+)")
var triggerDel = regexp.MustCompile("!trigger del ([a-zA-Z0-9_]+)")
var triggerEmote = regexp.MustCompile("^:([a-zA-Z0-9_]+):$")

type triggered struct {
	spaces sync.Map
	store  store.Store
}

type spaceTrigger struct {
	space    string
	store    store.Store
	triggers sync.Map
}

type trigger struct {
	Space   string `json:"space"`
	Keyword string `json:"keyword"`
	Value   string `json:"value"`
}

func (t *spaceTrigger) save(space string, trigg *trigger) error {
	triggers := make([]trigger, 0)

	if trigg != nil {
		t.triggers.Store(trigg.Keyword, *trigg)
	}
	t.triggers.Range(func(key interface{}, value interface{}) bool {
		triggers = append(triggers, value.(trigger))
		return true
	})
	return t.store.Collection("triggered-"+space).Set("triggered", triggers)
}

func (t *spaceTrigger) load(space string) error {
	triggers := make([]trigger, 0)
	if err := t.store.Collection("triggered-"+space).Get("triggered", &triggers); err != nil {
		return err
	}

	for _, trig := range triggers {
		t.triggers.Store(trig.Keyword, trig)
	}

	return nil
}

func (t *spaceTrigger) delete(space string, key string) error {
	t.triggers.Delete(key)
	return t.save(space, nil)
}

func (t *triggered) find(space string) *spaceTrigger {
	ts, _ := t.spaces.LoadOrStore(space, &spaceTrigger{space: space, store: t.store})
	return ts.(*spaceTrigger)
}

func (t *triggered) handleTriggerAdd(i instance.Instance, post *model.Post) error {
	space, err := i.SpaceOf(post.ChannelId)
	if err != nil {
		return err
	}
	subs := triggerAdd.FindStringSubmatch(post.Message)

	if len(subs) != 3 {
		i.Client().CreatePost(&model.Post{Message: "je comprends pô", RootId: post.Id, ChannelId: post.ChannelId})
		return nil
	}

	return t.find(space).save(space, &trigger{Keyword: strings.ToLower(subs[1]), Value: subs[2], Space: space})
}

func (t *triggered) handleTriggerDel(i instance.Instance, post *model.Post) error {
	subs := triggerDel.FindStringSubmatch(post.Message)

	if len(subs) != 2 {
		i.Client().CreatePost(&model.Post{Message: "t’y es preeeeesque", RootId: post.Id, ChannelId: post.ChannelId})
		return nil
	}

	space, err := i.SpaceOf(post.ChannelId)
	if err != nil {
		return nil
	}
	t.find(space).delete(space, subs[1])
	i.Client().CreatePost(&model.Post{Message: "Caroline… supprimée.", RootId: post.Id, ChannelId: post.ChannelId})
	return nil
}

func (t *triggered) handleTriggerList(i instance.Instance, post *model.Post) error {
	space, err := i.SpaceOf(post.ChannelId)
	if err != nil {
		return err
	}
	trigs := make([]string, 0)
	t.find(space).triggers.Range(func(key interface{}, value interface{}) bool {
		trigs = append(trigs, fmt.Sprintf(" * `%s`: %s", key, value.(trigger).Value))
		return true
	})

	msg := "Ah, ben yen a pas.\n\n * `!trigger add <nom> <ce que tu veux>`\n * `!trigger del <nom>`\n * `!trigger list`\n"
	if len(trigs) > 0 {
		msg = "Liste des :triggered:\n\n"
		msg = msg + strings.Join(trigs, "\n")
	}

	i.Client().CreatePost(&model.Post{Message: msg, ChannelId: post.ChannelId})

	return nil
}

func (t *triggered) handleMessage(i instance.Instance, post *model.Post) error {
	space, err := i.SpaceOf(post.ChannelId)
	if err != nil {
		return err
	}
	t.find(space).triggers.Range(func(key interface{}, value interface{}) bool {
		if strings.Contains(strings.ToLower(post.Message), fmt.Sprintf("*%s*", key)) {
			i.Client().CreatePost(&model.Post{Message: fmt.Sprintf("%s", value.(trigger).Value), ChannelId: post.ChannelId})
			return false
		}

		subs := triggerEmote.FindStringSubmatch(strings.ToLower(post.Message))

		if len(subs) == 2 && fmt.Sprintf("%s", key) == subs[1] {
			i.Client().CreatePost(&model.Post{Message: fmt.Sprintf("%s", value.(trigger).Value), ChannelId: post.ChannelId})
			return false
		}
		return true
	})
	return nil
}

func (t *triggered) Handler(i instance.Instance, event *model.WebSocketEvent) error {
	if post, err := helpers.DecodePost(event); err != nil {
		return nil
	} else if post != nil {
		if strings.HasPrefix(post.Message, "!trigger add ") {
			return t.handleTriggerAdd(i, post)
		} else if strings.HasPrefix(post.Message, "!trigger del") {
			return t.handleTriggerDel(i, post)
		} else if strings.HasPrefix(post.Message, "!trigger list") {
			return t.handleTriggerList(i, post)
		} else {
			return t.handleMessage(i, post)
		}
	}
	return nil
}

func NewTriggered(i instance.Instance, dbroot string) instance.Handler {
	teams, err := i.Client().GetTeamsForUser(i.Me().Id, "")
	if err.Error != nil {
		panic(err)
	}

	s := store.NewJSONStore(dbroot)

	t := &triggered{
		store: s,
	}

	for _, team := range teams {
		if err := t.find(team.Id).load(team.Id); err != nil {
			panic(err)
		}
	}

	return t.Handler
}
