package v1

import (
	"encoding/binary"
	"errors"
	"fmt"
	"math"
	"net"
	"sync"
	"time"

	"github.com/ahriroot/ahrikv/clients/golang/v1/command"
)

const (
	MAGIC_NUMBER_0 = 0x06
	MAGIC_NUMBER_1 = 0x06
	VERSION        = 0x01
)

type ChanMessage struct {
	value interface{}
	err   error
}

func NewAhrikvClient(config Config) (*Ahrikv, error) {
	return NewAhrikv(config, "default")
}

func NewAhrikv(config Config, database string) (*Ahrikv, error) {
	client := &Ahrikv{
		config:      config,
		database:    database,
		pendingReqs: make(map[uint32]chan ChanMessage),
		seq:         0,
	}

	if err := client.reconnect(); err != nil {
		return nil, err
	}

	return client, nil
}

type Ahrikv struct {
	config      Config
	conn        net.Conn
	database    string
	mu          sync.Mutex
	closed      bool
	pendingReqs map[uint32]chan ChanMessage
	seq         uint32
}

func (c *Ahrikv) reconnect() error {
	if c.conn != nil {
		c.conn.Close()
		c.conn = nil
	}

	// 重新建立连接
	addr := net.JoinHostPort(c.config.Host, fmt.Sprintf("%d", c.config.Port))
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		return fmt.Errorf("reconnect failed: %v", err)
	}

	c.conn = conn

	// 重新认证
	if err := c.authenticate(conn); err != nil {
		conn.Close()
		return fmt.Errorf("re-authenticate failed: %v", err)
	}

	go c.recv()

	go func() {
		for {
			if _, err := c.ping(); err != nil {
				c.conn.Close()
				break
			}
			time.Sleep(time.Second * 2)
		}
	}()

	return nil
}

func (c *Ahrikv) authenticate(conn net.Conn) error {
	header := make([]byte, 12)
	header[0], header[1] = MAGIC_NUMBER_0, MAGIC_NUMBER_1
	header[2] = VERSION
	header[3] = CmdAuthenticate

	cmd := command.Authenticate{
		Secret: c.config.Secret,
	}

	body, err := cmd.Serialize()
	if err != nil {
		return err
	}

	bodyLen := uint32(len(body))
	binary.BigEndian.PutUint32(header[4:8], 0)
	binary.BigEndian.PutUint32(header[8:12], bodyLen)

	if _, err := conn.Write(header); err != nil {
		return err
	}
	if _, err := conn.Write(body); err != nil {
		return err
	}

	header = make([]byte, 12)
	if _, err := conn.Read(header); err != nil {
		return err
	}
	if header[0] != MAGIC_NUMBER_0 || header[1] != MAGIC_NUMBER_1 {
		return fmt.Errorf("invalid magic number")
	}
	if header[2] != VERSION {
		return fmt.Errorf("invalid version")
	}
	if header[3] != CmdAuthenticate {
		return fmt.Errorf("invalid response type")
	}
	bodyLen = binary.BigEndian.Uint32(header[8:12])
	body = make([]byte, bodyLen)
	if _, err := conn.Read(body); err != nil {
		return err
	}
	rs, err := command.DeserializeResultAuthenticate(body)
	if err != nil {
		return err
	}
	if !rs.Ok {
		return errors.New(rs.Msg)
	}

	return nil
}

func (c *Ahrikv) checkConnection() error {
	c.mu.Lock()
	defer c.mu.Unlock()

	if c.closed {
		return errors.New("client is closed")
	}

	// 简单检查连接是否活跃
	if c.conn == nil {
		return c.reconnect()
	}

	// 发送PING命令或空数据检查连接状态
	_, err := c.conn.Write([]byte{})
	if err != nil {
		return c.reconnect()
	}

	return nil
}

func (a *Ahrikv) getSeq() uint32 {
	a.mu.Lock()
	defer a.mu.Unlock()

	if a.seq == math.MaxUint32 {
		a.seq = 1
	} else {
		a.seq++
	}
	return a.seq
}

func (a *Ahrikv) ping() (*command.ResultPing, error) {
	cmd := command.Ping{}
	rs, err := a.send(CmdPing, cmd)
	if err != nil {
		a.conn.Close()
		return nil, err
	}
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultPing), nil
}

func (a *Ahrikv) send(cmdType uint8, cmd command.Command) (chan ChanMessage, error) {
	if err := a.checkConnection(); err != nil {
		return nil, err
	}

	ch := make(chan ChanMessage)
	var id uint32 = a.getSeq()
	a.pendingReqs[id] = ch

	header := make([]byte, 12)
	header[0], header[1] = MAGIC_NUMBER_0, MAGIC_NUMBER_1
	header[2] = VERSION
	header[3] = cmdType

	body, err := cmd.Serialize()
	if err != nil {
		return nil, err
	}
	bodyLen := uint32(len(body))

	// 写入序列号
	binary.BigEndian.PutUint32(header[4:8], id)
	// 写入长度 (大端序)
	binary.BigEndian.PutUint32(header[8:12], bodyLen)

	// 发送头部 + 请求体
	_, err = a.conn.Write(header)
	if err != nil {
		return nil, err
	}
	_, err = a.conn.Write(body)
	if err != nil {
		return nil, err
	}
	return ch, nil
}

func (a *Ahrikv) recv() error {
	for {
		header := make([]byte, 12)
		if _, err := a.conn.Read(header); err != nil {
			continue
		}
		if header[0] != MAGIC_NUMBER_0 || header[1] != MAGIC_NUMBER_1 {
			continue
		}
		if header[2] != VERSION {
			continue
		}
		seq := binary.BigEndian.Uint32(header[4:8])
		ch, ok := a.pendingReqs[seq]
		if !ok {
			continue
		}
		bodyLen := binary.BigEndian.Uint32(header[8:12])
		body := make([]byte, bodyLen)
		if _, err := a.conn.Read(body); err != nil {
			continue
		}
		switch header[3] {
		case CmdPing:
			rs, err := command.DeserializeResultPing(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdKeys:
			rs, err := command.DeserializeResultKeys(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdExists:
			rs, err := command.DeserializeResultExistsString(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdExpire:
			rs, err := command.DeserializeResultExpireString(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdSetString:
			rs, err := command.DeserializeResultSetString(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdGetString:
			rs, err := command.DeserializeResultGetString(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdDelString:
			rs, err := command.DeserializeResultDelString(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdHashSet:
			rs, err := command.DeserializeResultHashSet(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdHashGet:
			rs, err := command.DeserializeResultHashGet(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdHashDel:
			rs, err := command.DeserializeResultHashDel(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdHashExists:
			rs, err := command.DeserializeResultHashExists(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdHashLen:
			rs, err := command.DeserializeResultHashLen(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case CmdHashFields:
			rs, err := command.DeserializeResultHashFields(body)
			ch <- ChanMessage{
				value: rs,
				err:   err,
			}
		case Error:
			rs, err := command.DeserializeResultError(body)
			if err != nil {
				ch <- ChanMessage{
					value: rs,
					err:   err,
				}
			} else {
				err = errors.New(rs.Msg)
				ch <- ChanMessage{
					value: rs,
					err:   err,
				}
			}
		default:
			return fmt.Errorf("invalid response type")
		}
	}
}

func (a *Ahrikv) Close() error {
	a.mu.Lock()
	defer a.mu.Unlock()

	if a.closed {
		return nil
	}

	a.closed = true
	if a.conn != nil {
		return a.conn.Close()
	}
	return nil
}

// With: Create a new Ahrikv client with a different database.
//
// Params:
//
//	database: string - The name of the database to use.
//
// Returns:
//
//	*Ahrikv - A new Ahrikv client with the specified database.
//
// Example:
//
//	client := NewAhrikvClient(Config{Host: "127.0.0.1", Port: 8080, Secret: "mysecret"})
//	db1 := client.With("db1")
func (a *Ahrikv) With(database string) *Ahrikv {
	return &Ahrikv{
		config:   a.config,
		conn:     a.conn,
		database: database,
	}
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
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultKeys), nil
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
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultExistsString), nil
}

// Expire: Set expire time for a key.
//
// Params:
//
//	key: string - The key to set expire time for.
//	ttl: *uint64 - The expire time in seconds. If it is 0, the key will not expire.
//
// Returns:
//
//	*command.ResultExpireString - The result of the operation.
//
// Example:
//
//	client := NewAhrikvClient(Config{Host: "127.0.0.1", Port: 8080, Secret: "mysecret"})
//	result, err := client.Expire("mykey", 10)
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
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultExpireString), nil
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
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultSetString), nil
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
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultGetString), nil
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
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultDelString), nil
}

func (a *Ahrikv) HashSet(key string, field string, value string, ttl ...uint64) (*command.ResultHashSet, error) {
	cmd := command.HashSet{
		DB:     a.database,
		Key:    key,
		Field:  field,
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

	rs, err := a.send(CmdHashSet, cmd)

	if err != nil {
		return nil, err
	}
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultHashSet), nil
}

func (a *Ahrikv) HashGet(key string, field string) (*command.ResultHashGet, error) {
	cmd := command.HashGet{
		DB:    a.database,
		Key:   key,
		Field: field,
	}

	rs, err := a.send(CmdHashGet, cmd)
	if err != nil {
		return nil, err
	}
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultHashGet), nil
}

func (a *Ahrikv) HashDel(key string, field string) (*command.ResultHashDel, error) {
	cmd := command.HashDel{
		DB:    a.database,
		Key:   key,
		Field: field,
	}

	rs, err := a.send(CmdHashDel, cmd)
	if err != nil {
		return nil, err
	}
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultHashDel), nil
}

func (a *Ahrikv) HashExists(key string, field string) (*command.ResultHashExists, error) {
	cmd := command.HashExists{
		DB:    a.database,
		Key:   key,
		Field: field,
	}

	rs, err := a.send(CmdHashExists, cmd)
	if err != nil {
		return nil, err
	}
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultHashExists), nil
}

func (a *Ahrikv) HashLen(key string) (*command.ResultHashLen, error) {
	cmd := command.HashLen{
		DB:  a.database,
		Key: key,
	}

	rs, err := a.send(CmdHashLen, cmd)
	if err != nil {
		return nil, err
	}
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultHashLen), nil
}

func (a *Ahrikv) HashFields(key string) (*command.ResultHashFields, error) {
	cmd := command.HashFields{
		DB:  a.database,
		Key: key,
	}

	rs, err := a.send(CmdHashFields, cmd)
	if err != nil {
		return nil, err
	}
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultHashFields), nil
}
