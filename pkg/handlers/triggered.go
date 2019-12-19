package handlers

import (
	"flobot/pkg/helpers"
	"flobot/pkg/instance/mattermost"
	"fmt"
	"regexp"
	"sort"
	"strings"
	"sync"

	"flobot/pkg/instance"
	"flobot/pkg/store"

	"github.com/mattermost/mattermost-server/model"
)

var triggerAdd = regexp.MustCompile("^!trigger\\s+([a-z]+)\\s+\"([^\"]+)\"\\s+(.+)$")
var triggerDel = regexp.MustCompile("^!trigger del \"([^\"]+)\"$")

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
	Space    string `json:"space"`
	Keyword  string `json:"keyword"`
	Value    string `json:"value"`
	Reaction bool   `json:"reaction,omitempty"`
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

func (t *triggered) handleTriggerAdd(c instance.Channel, post model.Post) error {
	space, err := c.Space()
	if err != nil {
		return err
	}
	subs := triggerAdd.FindStringSubmatch(post.Message)

	if len(subs) != 4 {
		return helpers.Discard(c.Reply(post, "lapô compri, lapô compri."))
	}

	trig := &trigger{
		Keyword: strings.ToLower(subs[2]),
		Value:   subs[3],
		Space:   space,
	}

	if subs[1] == "reaction" {
		trig.Reaction = true
		trig.Value = subs[3]
	} else if subs[1] == "text" {
		trig.Reaction = false
	} else {
		return helpers.Discard(c.Reply(post, "nah spô comme ça : `!trigger text|reaction \"trigger\" value"))
	}

	if err := t.find(space).save(space, trig); err != nil {
		return c.React(post, "x")
	}

	return c.React(post, "white_check_mark")
}

func (t *triggered) handleTriggerDel(c instance.Channel, post model.Post) error {
	subs := triggerDel.FindStringSubmatch(post.Message)

	if len(subs) != 2 {
		return helpers.Discard(c.Post(model.Post{Message: "t’y es preeeeesque", RootId: post.Id, ChannelId: post.ChannelId}))
	}

	space, err := c.Space()
	if err != nil {
		return nil
	}
	t.find(space).delete(space, subs[1])
	return helpers.Discard(c.Post(model.Post{Message: "Caroline… supprimée.", RootId: post.Id, ChannelId: post.ChannelId}))
}

func (t *triggered) handleTriggerList(c instance.Channel, post model.Post) error {
	space, err := c.Space()
	if err != nil {
		return err
	}
	trigs := make([]string, 0)
	t.find(space).triggers.Range(func(key interface{}, value interface{}) bool {
		trigs = append(trigs, fmt.Sprintf(" * `%s`: %s", key, value.(trigger).Value))
		return true
	})

	msg := "Ah, ben yen a pô."
	if len(trigs) > 0 {
		sort.Strings(trigs)
		msg = fmt.Sprintf("Liste des %d triggers :triggered: :\n\n", len(trigs))
		msg = msg + strings.Join(trigs, "\n")
	}

	return helpers.Discard(c.Post(model.Post{Message: msg, ChannelId: post.ChannelId}))
}

func (t *triggered) handleMessage(c instance.Channel, post model.Post) error {
	space, err := c.Space()
	if err != nil {
		return err
	}
	msg := strings.ToLower(post.Message)

	var fval string
	var reaction bool

	t.find(space).triggers.Range(func(key interface{}, value interface{}) bool {
		c1 := fmt.Sprintf("%s ", key)
		c2 := fmt.Sprintf(" %s ", key)
		c3 := fmt.Sprintf(" %s", key)
		c4 := strings.TrimSpace(key.(string))
		c5 := ":" + c4 + ":"

		if strings.Contains(msg, c2) || strings.HasPrefix(msg, c1) || strings.HasSuffix(msg, c3) || key.(string) == msg || msg == c4 || msg == c5 {
			fval = value.(trigger).Value
			reaction = value.(trigger).Reaction
			return false
		}
		return true
	})

	if fval == "" {
		return nil
	}

	if !reaction {
		return helpers.Discard(c.PostOrReply(post, fval))
	}
	return c.React(post, fval)
}

func (t *triggered) Handler(i instance.Instance, event model.WebSocketEvent) error {
	if post, err := mattermost.DecodePost(event); err != nil {
		return nil
	} else if post != nil {
		c := i.Client().Chan.Get(post.ChannelId)
		if triggerAdd.MatchString(post.Message) {
			return t.handleTriggerAdd(c, *post)
		} else if strings.HasPrefix(post.Message, "!trigger del") {
			return t.handleTriggerDel(c, *post)
		} else if strings.HasPrefix(post.Message, "!trigger list") {
			return t.handleTriggerList(c, *post)
		} else if strings.HasPrefix(post.Message, "!trigger ") {
			return helpers.Discard(c.Reply(*post, "wut?"))
		} else {
			return t.handleMessage(c, *post)
		}
	}
	return nil
}

func NewTriggered(i instance.Instance) instance.Handler {
	spaces, err := i.Client().Me.Spaces()
	if err != nil {
		panic(err)
	}
	t := &triggered{
		store: i.Store(),
	}

	for _, space := range spaces {
		if err := t.find(space).load(space); err != nil {
			panic(err)
		}
	}

	return t.Handler
}
