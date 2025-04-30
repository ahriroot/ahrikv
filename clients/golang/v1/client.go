package v1

import (
	"encoding/binary"
	"fmt"
	"net"

	command_string "github.com/ahriroot/ahrikv/clients/golang/v1/command"
)

func NewAhrikv(config Config) (*Ahrikv, error) {
	addr := net.JoinHostPort(config.Host, fmt.Sprintf("%d", config.Port))
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		return nil, err
	}

	return &Ahrikv{
		config: config,
		conn:   conn,
	}, nil
}

type Ahrikv struct {
	config Config
	conn   net.Conn
}

func (a *Ahrikv) Set(key, value string, ttl ...uint64) (string, error) {
	header := make([]byte, 8)
	header[0], header[1] = 0x06, 0x06 // 魔数
	header[2] = 1                     // 版本
	header[3] = 2                     // 类型 (示例: 2=SET命令)

	cmd := command_string.GetString{
		DB:  "default",
		Key: key,
	}
	// 准备请求体
	body := []byte("hello rust")
	bodyLen := uint32(len(body))

	// 写入长度 (大端序)
	binary.BigEndian.PutUint32(header[4:8], bodyLen)

	// 发送头部 + 请求体
	a.conn.Write(header)
	a.conn.Write(body)
	return "", nil
}

func (a *Ahrikv) Get(key string) (string, uint64, error) {
	return "", 0, nil
}
