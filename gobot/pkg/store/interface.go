package store

type Store interface {
	Collection(name string) Collection
}

type Collection interface {
	Set(key string, value interface{}) error
	Get(key string, out interface{}) error
	Del(key string) error
}
