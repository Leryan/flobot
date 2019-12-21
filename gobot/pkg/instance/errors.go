package instance

import "fmt"

type Error struct {
	Code   int
	Status string
}

func (e Error) Error() string {
	return fmt.Sprintf("code %d status: %s", e.Code, e.Status)
}
