package v1

import (
	"time"
)

type Mode int

const (
	Active  Mode = 1
	Passive Mode = 2
)

type Config struct {
	Host                 string
	Port                 int
	Secret               string
	Mode                 Mode
	PingInterval         time.Duration
	ReconnectInterval    time.Duration
	MaxReconnectAttempts int
}

func NewConfig() *Config {
	return &Config{
		Host:                 "127.0.0.1",
		Port:                 6000,
		Secret:               "",
		Mode:                 Active,
		PingInterval:         60 * time.Second,
		ReconnectInterval:    5 * time.Second,
		MaxReconnectAttempts: 0,
	}
}
