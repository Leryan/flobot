package main

import (
	"flobot/pkg/conf"
	"flobot/pkg/handlers"
	"flobot/pkg/instance"
	"flobot/pkg/middlewares"

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
			log.Printf(
				"exit with: %v",
				instance.NewFromCfg(cfg).
					AddMiddleware(middlewares.Security).
					AddHandler(handlers.Console).
					AddHandler(handlers.Parrot).
					AddHandler(handlers.Avis).
					AddHandler(handlers.NewWerewolf().Handle).
					AddHandler(handlers.DisBonjour).
					AddHandler(handlers.EmmerdeMaison).
					AddHandler(handlers.HyperCon).
					AddHandler(handlers.NewTriggered("prout")).
					Run(),
			)
		}(cfg)
	}
	wg.Wait()
}
