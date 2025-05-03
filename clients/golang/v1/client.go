package v1

import (
	"encoding/binary"
	"errors"
	"fmt"
	"net"

	"github.com/ahriroot/ahrikv/clients/golang/v1/command"
)

const (
	MAGIC_NUMBER_0 = 0x06
	MAGIC_NUMBER_1 = 0x06
	VERSION        = 0x01
)

func NewAhrikvClient(config Config) (*Ahrikv, error) {
	return NewAhrikv(config, "default")
}

func NewAhrikv(config Config, database string) (*Ahrikv, error) {
	addr := net.JoinHostPort(config.Host, fmt.Sprintf("%d", config.Port))
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		return nil, err
	}

	header := make([]byte, 8)
	header[0], header[1] = MAGIC_NUMBER_0, MAGIC_NUMBER_1
	header[2] = VERSION
	header[3] = CmdAuthenticate

	cmd := command.Authenticate{
		Secret: config.Secret,
	}
	// 准备请求体
	body, err := cmd.Serialize()
	if err != nil {
		return nil, err
	}
	bodyLen := uint32(len(body))

	// 写入长度 (大端序)
	binary.BigEndian.PutUint32(header[4:8], bodyLen)

	// 发送头部 + 请求体
	conn.Write(header)
	conn.Write(body)

	// 读取响应
	header = make([]byte, 8)
	if _, err := conn.Read(header); err != nil {
		return nil, err
	}
	if header[0] != MAGIC_NUMBER_0 || header[1] != MAGIC_NUMBER_1 {
		return nil, fmt.Errorf("invalid magic number")
	}
	if header[2] != VERSION {
		return nil, fmt.Errorf("invalid version")
	}
	if header[3] != CmdAuthenticate {
		return nil, fmt.Errorf("invalid response type")
	}
	bodyLen = binary.BigEndian.Uint32(header[4:8])
	body = make([]byte, bodyLen)
	if _, err := conn.Read(body); err != nil {
		return nil, err
	}
	rs, err := command.DeserializeResultAuthenticate(body)
	if err != nil {
		return nil, err
	}
	if !rs.Ok {
		return nil, errors.New(rs.Msg)
	}

	return &Ahrikv{
		config:   config,
		conn:     conn,
		database: database,
	}, nil
}

type Ahrikv struct {
	config   Config
	conn     net.Conn
	database string
}

func (a *Ahrikv) send(cmdType uint8, cmd command.Command) (interface{}, error) {
	header := make([]byte, 8)
	header[0], header[1] = MAGIC_NUMBER_0, MAGIC_NUMBER_1
	header[2] = VERSION
	header[3] = cmdType

	body, err := cmd.Serialize()
	if err != nil {
		return nil, err
	}
	bodyLen := uint32(len(body))

	// 写入长度 (大端序)
	binary.BigEndian.PutUint32(header[4:8], bodyLen)

	// 发送头部 + 请求体
	a.conn.Write(header)
	a.conn.Write(body)

	// 读取响应
	header = make([]byte, 8)
	if _, err := a.conn.Read(header); err != nil {
		return nil, err
	}
	if header[0] != MAGIC_NUMBER_0 || header[1] != MAGIC_NUMBER_1 {
		return nil, fmt.Errorf("invalid magic number")
	}
	if header[2] != VERSION {
		return nil, fmt.Errorf("invalid version")
	}
	if header[3] != cmdType {
		return nil, fmt.Errorf("invalid response type")
	}
	bodyLen = binary.BigEndian.Uint32(header[4:8])
	body = make([]byte, bodyLen)
	if _, err := a.conn.Read(body); err != nil {
		return nil, err
	}
	switch cmdType {
	case CmdKeys:
		rs, err := command.DeserializeResultKeys(body)
		if err != nil {
			return nil, err
		}
		return rs, nil
	case CmdExists:
		rs, err := command.DeserializeResultExistsString(body)
		if err != nil {
			return nil, err
		}
		return rs, nil
	case CmdExpire:
		rs, err := command.DeserializeResultExpireString(body)
		if err != nil {
			return nil, err
		}
		return rs, nil
	case CmdSetString:
		rs, err := command.DeserializeResultSetString(body)
		if err != nil {
			return nil, err
		}
		return rs, nil
	case CmdGetString:
		rs, err := command.DeserializeResultGetString(body)
		if err != nil {
			return nil, err
		}
		return rs, nil
	case CmdDelString:
		rs, err := command.DeserializeResultDelString(body)
		if err != nil {
			return nil, err
		}
		return rs, nil
	default:
		return nil, fmt.Errorf("invalid response type")
	}
}

func (a *Ahrikv) Close() error {
	return a.conn.Close()
}

func (a *Ahrikv) Keys(pattern string) (*command.ResultKeys, error) {
	cmd := command.Keys{
		DB:   a.database,
		Page: 1,
		Size: 10,
	}

	rs, err := a.send(CmdKeys, cmd)
	if err != nil {
		return nil, err
	}
	return rs.(*command.ResultKeys), nil
}

func (a *Ahrikv) Exists(key string) (*command.ResultExistsString, error) {
	cmd := command.ExistsString{
		DB:  a.database,
		Key: key,
	}

	rs, err := a.send(CmdExists, cmd)
	if err != nil {
		return nil, err
	}
	return rs.(*command.ResultExistsString), nil
}

func (a *Ahrikv) Expire(key string, ttl *uint64) (*command.ResultExpireString, error) {
	cmd := command.ExpireString{
		DB:     a.database,
		Key:    key,
		Expire: ttl,
	}

	rs, err := a.send(CmdExpire, cmd)
	if err != nil {
		return nil, err
	}
	return rs.(*command.ResultExpireString), nil
}

func (a *Ahrikv) Set(key, value string, ttl ...uint64) (*command.ResultSetString, error) {
	cmd := command.SetString{
		DB:     a.database,
		Key:    key,
		Value:  value,
		Expire: nil,
	}
	if len(ttl) > 0 {
		if ttl[0] == 0 {
			cmd.Expire = nil
		} else {
			cmd.Expire = &ttl[0]
		}
	}

	rs, err := a.send(CmdSetString, cmd)
	if err != nil {
		return nil, err
	}
	return rs.(*command.ResultSetString), nil
}

func (a *Ahrikv) Get(key string) (*command.ResultGetString, error) {
	cmd := command.GetString{
		DB:  a.database,
		Key: key,
	}

	rs, err := a.send(CmdGetString, cmd)
	if err != nil {
		return nil, err
	}
	return rs.(*command.ResultGetString), nil
}

func (a *Ahrikv) Del(key string) (*command.ResultDelString, error) {
	cmd := command.DelString{
		DB:  a.database,
		Key: key,
	}

	rs, err := a.send(CmdDelString, cmd)
	if err != nil {
		return nil, err
	}
	return rs.(*command.ResultDelString), nil
}
