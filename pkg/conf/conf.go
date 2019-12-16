package conf

import "os"

type Instance struct {
	// Name of the bot, as display name.
	Name     string
	TeamName string
	// DebugChan returns the ID of an existing chan where to send debug messages.
	// If empty, no debugging.
	DebugChan string
	APIv4URL  string
	// WS websocket url
	WS    string
	Token string
}

func Instances() []Instance {
	var insts []Instance

	insts = append(insts, Instance{
		Name:      os.Getenv("BOT_NAME"),
		DebugChan: os.Getenv("BOT_DEBUG_CHAN"),
		APIv4URL:  os.Getenv("BOT_APIV4URL"),
		Token:     os.Getenv("BOT_TOKEN"),
		WS:        os.Getenv("BOT_WS"),
		TeamName:  os.Getenv("BOT_TEAM_NAME"),
	})

	return insts
}
