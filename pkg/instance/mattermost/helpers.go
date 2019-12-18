package mattermost

import (
	"flobot/pkg/instance"

	"github.com/mattermost/mattermost-server/model"
)

func ToError(resp *model.Response) error {
	if resp.Error == nil {
		return nil
	}

	return instance.Error{
		Code:   resp.StatusCode,
		Status: resp.Error.ToJson(),
	}
}
