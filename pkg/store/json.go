package store

import (
	"encoding/json"
	"fmt"
	"os"
	"path"

	"github.com/pkg/errors"
)

type jsonStore struct {
	root string
}

func (s *jsonStore) Collection(name string) Collection {
	return &jsonCollection{name: name, root: s.root}
}

func NewJSONStore(root string) Store {
	_, err := os.Stat(root)
	if os.IsNotExist(err) {
		os.MkdirAll(root, 0750)
	}
	return &jsonStore{root: root}
}

type jsonCollection struct {
	name string
	root string
}

func (c *jsonCollection) collkey(key string) string {
	return path.Join(c.root, fmt.Sprintf("%s-%s", c.name, key))
}

func (c *jsonCollection) Set(key string, value interface{}) error {
	f, err := os.OpenFile(c.collkey(key), os.O_WRONLY|os.O_CREATE, 0640)
	if err != nil {
		return errors.Wrap(err, "open store file.Set")
	}
	defer f.Close()
	return errors.Wrap(json.NewEncoder(f).Encode(value), "encode key in collection "+c.name)
}

func (c *jsonCollection) Get(key string, out interface{}) error {
	f, err := os.Open(c.collkey(key))
	if os.IsNotExist(err) {
		return nil
	}
	if err != nil {
		return errors.Wrap(err, "open store file.Get")
	}
	defer f.Close()
	return json.NewDecoder(f).Decode(out)
}

func (c *jsonCollection) Del(key string) error {
	_, err := os.Stat(c.collkey(key))
	if os.IsNotExist(err) {
		return nil
	}
	return os.Remove(c.collkey(key))
}
