package main

import (
	"flobot/pkg/conf"
	"flobot/pkg/handlers"
	"flobot/pkg/instance"
	"flobot/pkg/instance/mattermost"
	"flobot/pkg/middlewares"
	store2 "flobot/pkg/store"
	"fmt"
	"runtime/debug"

	"github.com/mattermost/mattermost-server/model"

	"log"
	"sync"
)

func main() {
	store := store2.NewJSONStore("botdb")
	wg := sync.WaitGroup{}
	for i, cfg := range conf.Instances() {
		log.Printf("spawning instance number %d", i)
		wg.Add(1)
		go func(cfg conf.Instance) {
			defer wg.Done()

			var inst instance.Instance

			defer func() {
				msg := "/quit"
				if r := recover(); r != nil {
					log.Printf("paniced: %v", r)
					debug.PrintStack()
					msg = fmt.Sprintf("panic: %v", r)

				}
				inst.Client().Chan.Get(inst.Cfg().DebugChan).Post(
					model.Post{Message: msg},
				)
			}()

			inst = mattermost.NewMattermost(cfg, store)
			log.Printf(
				"exit with: %v",
				inst.AddMiddleware(middlewares.Security).
					AddHandler(handlers.Console).
					AddHandler(handlers.Parrot).
					//AddHandler(handlers.NewWerewolf().Handle).
					AddHandler(handlers.DisBonjour).
					AddHandler(handlers.EmmerdeMaison).
					AddHandler(handlers.HyperCon).
					AddHandler(handlers.NewProg().Handle).
					AddHandler(handlers.NewTriggered(inst)).
					Run(),
			)
		}(cfg)
	}
	wg.Wait()
}
