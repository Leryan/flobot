package main

import (
	"flobot/pkg/conf"
	"flobot/pkg/handlers"
	"flobot/pkg/instance"
)

func main() {
	i := instance.NewFromCfg(conf.Instances()[0])
	i.AddHandler(handlers.Console)
	i.AddHandler(handlers.Parrot)
	i.Run()
}
