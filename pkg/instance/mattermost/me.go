package mattermost

import "github.com/mattermost/mattermost-server/model"

type Me struct {
	client *model.Client4
}

func (m *Me) Me() (*model.User, error) {
	user, resp := m.client.GetMe("")
	if err := ToError(resp); err != nil {
		return nil, err
	}
	return user, nil
}

func (m *Me) Spaces() ([]string, error) {
	var spaces []string
	if me, err := m.Me(); err != nil {
		return nil, err
	} else {
		teams, resp := m.client.GetTeamsForUser(me.Id, "")
		if err := ToError(resp); err != nil {
			return nil, err
		}
		for _, team := range teams {
			spaces = append(spaces, team.Id)
		}
	}

	return spaces, nil
}
