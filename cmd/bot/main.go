package main

import (
	"flobot/pkg/conf"
	"flobot/pkg/handlers"
	"flobot/pkg/instance"
	"flobot/pkg/middlewares"
	"fmt"
	"runtime/debug"

	"github.com/mattermost/mattermost-server/model"

	"log"
	"sync"
)

func main() {
	wg := sync.WaitGroup{}
	for i, cfg := range conf.Instances() {
		log.Printf("spawning instance number %d", i)
		wg.Add(1)
		go func(cfg conf.Instance) {
			defer wg.Done()

			var inst instance.Instance

			defer func() {
				if r := recover(); r != nil {
					log.Printf("paniced: %v", r)
					debug.PrintStack()
					inst.Client().CreatePost(&model.Post{ChannelId: inst.Cfg().DebugChan, Message: fmt.Sprintf("panic: %v", r)})
				}
			}()

			inst = instance.NewMattermost(cfg)
			log.Printf(
				"exit with: %v",
				inst.AddMiddleware(middlewares.Security).
					AddHandler(handlers.Console).
					AddHandler(handlers.Parrot).
					AddHandler(handlers.Avis).
					AddHandler(handlers.NewWerewolf().Handle).
					AddHandler(handlers.DisBonjour).
					AddHandler(handlers.EmmerdeMaison).
					AddHandler(handlers.HyperCon).
					AddHandler(handlers.NewTriggered(inst, "botdb")).
					Run(),
			)
		}(cfg)
	}
	wg.Wait()
}
