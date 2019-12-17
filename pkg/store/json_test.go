package store

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/google/uuid"
)

func TestJSONStore(t *testing.T) {
	assert := assert.New(t)
	s := NewJSONStore("testdata-" + uuid.New().String())
	c := s.Collection("someofit")

	c1 := make(map[string]string)
	c1["key"] = "value"

	assert.Nil(c.Get("somekey", &c1))

	assert.Contains(c1, "key")
	assert.Equal(c1["key"], "value")

	assert.Nil(c.Del("somekey"))
	assert.Nil(c.Set("somekey", c1))

	c2 := make(map[string]string)

	assert.Nil(c.Get("somekey", &c2))

	assert.Equal(c2["key"], "value")
}
