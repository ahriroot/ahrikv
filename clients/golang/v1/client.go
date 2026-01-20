package v1

import (
	"encoding/binary"
	"errors"
	"fmt"
	"log"
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
	if config.Mode != Active && config.Mode != Passive {
		config.Mode = Active
	}
	if config.PingInterval == 0 {
		config.PingInterval = 60 * time.Second
	}
	if config.ReconnectInterval == 0 {
		config.ReconnectInterval = 5 * time.Second
	}
	client := &Ahrikv{
		config:         config,
		database:       database,
		pendingReqs:    make(map[uint32]chan ChanMessage),
		seq:            0,
		running:        false,
		reconnectCount: 0,
	}
	return client, nil
}

type Ahrikv struct {
	config         Config
	conn           net.Conn
	database       string
	mu             sync.RWMutex
	closed         bool
	pendingReqs    map[uint32]chan ChanMessage
	seq            uint32
	running        bool
	reconnectCount int
}

func (a *Ahrikv) dial() error {
	addr := net.JoinHostPort(a.config.Host, fmt.Sprintf("%d", a.config.Port))
	conn, err := net.Dial("tcp", addr)
	if err != nil {
		return err
	}
	a.mu.Lock()
	a.conn = conn
	a.mu.Unlock()
	return nil
}

func (a *Ahrikv) authenticate() error {
	a.mu.RLock()
	conn := a.conn
	a.mu.RUnlock()

	if conn == nil {
		return errors.New("connection is nil")
	}

	header := make([]byte, 12)
	header[0], header[1] = MAGIC_NUMBER_0, MAGIC_NUMBER_1
	header[2] = VERSION
	header[3] = CmdAuthenticate

	cmd := command.Authenticate{
		Secret: a.config.Secret,
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

func (a *Ahrikv) startPingLoop() {
	for a.running {
		time.Sleep(a.config.PingInterval)
		a.mu.RLock()
		conn := a.conn
		a.mu.RUnlock()

		if conn == nil {
			continue
		}

		cmd := command.Ping{}
		messageBytes, err := cmd.Serialize()
		if err != nil {
			log.Printf("Failed to serialize ping: %v", err)
			a.handleDisconnect()
			continue
		}

		header := make([]byte, 12)
		header[0], header[1] = MAGIC_NUMBER_0, MAGIC_NUMBER_1
		header[2] = VERSION
		header[3] = CmdPing

		bodyLen := uint32(len(messageBytes))
		binary.BigEndian.PutUint32(header[4:8], 0)
		binary.BigEndian.PutUint32(header[8:12], bodyLen)

		if err := binary.Write(conn, binary.BigEndian, header); err != nil {
			log.Printf("Failed to send ping header: %v", err)
			a.handleDisconnect()
			continue
		}
		if _, err := conn.Write(messageBytes); err != nil {
			log.Printf("Failed to send ping: %v", err)
			a.handleDisconnect()
			continue
		}
	}
}

func (a *Ahrikv) handleDisconnect() {
	a.mu.Lock()
	if a.conn != nil {
		a.conn.Close()
		a.conn = nil
	}
	a.mu.Unlock()

	a.reconnectCount++
	if a.config.MaxReconnectAttempts > 0 && a.reconnectCount >= a.config.MaxReconnectAttempts {
		log.Printf("Max reconnection attempts (%d) reached, stopping", a.config.MaxReconnectAttempts)
		a.running = false
		return
	}

	log.Printf("Connection lost, attempting to reconnect... (attempt %d)", a.reconnectCount)

	for a.running {
		time.Sleep(a.config.ReconnectInterval)
		if !a.running {
			break
		}

		if err := a.dial(); err != nil {
			log.Printf("Reconnection failed: %v, retrying in %v...", err, a.config.ReconnectInterval)
			continue
		}

		if err := a.authenticate(); err != nil {
			log.Printf("Authentication failed: %v, retrying...", err)
			a.mu.Lock()
			if a.conn != nil {
				a.conn.Close()
				a.conn = nil
			}
			a.mu.Unlock()
			continue
		}

		log.Printf("Successfully reconnected after %d attempts", a.reconnectCount)
		a.reconnectCount = 0
		break
	}
}

func (a *Ahrikv) Connect(callback ...func(message interface{})) error {
	a.running = true
	a.reconnectCount = 0

	if err := a.dial(); err != nil {
		return fmt.Errorf("Failed to connect: %w", err)
	}

	if err := a.authenticate(); err != nil {
		return fmt.Errorf("Failed to authenticate: %w", err)
	}

	if a.config.PingInterval < time.Second*5 {
		a.config.PingInterval = time.Second * 5
	}

	go a.startPingLoop()
	go a.recv(callback...)
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
		a.mu.Lock()
		if a.conn != nil {
			a.conn.Close()
			a.conn = nil
		}
		a.mu.Unlock()
		return nil, err
	}
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultPing), nil
}

func (a *Ahrikv) send(cmdType uint8, cmd command.Command) (chan ChanMessage, error) {
	a.mu.RLock()
	conn := a.conn
	a.mu.RUnlock()

	if conn == nil {
		return nil, errors.New("not connected to server")
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
	binary.BigEndian.PutUint32(header[4:8], id)
	binary.BigEndian.PutUint32(header[8:12], bodyLen)

	_, err = conn.Write(header)
	if err != nil {
		return nil, err
	}
	_, err = conn.Write(body)
	if err != nil {
		return nil, err
	}

	return ch, nil
}

func (a *Ahrikv) recv(callback ...func(message interface{})) {
	for a.running {
		a.mu.RLock()
		conn := a.conn
		a.mu.RUnlock()

		if conn == nil {
			time.Sleep(100 * time.Millisecond)
			continue
		}

		header := make([]byte, 12)
		if _, err := conn.Read(header); err != nil {
			log.Printf("Read header error: %v", err)
			a.handleDisconnect()
			continue
		}

		if header[0] != MAGIC_NUMBER_0 || header[1] != MAGIC_NUMBER_1 {
			log.Printf("Invalid magic number")
			a.handleDisconnect()
			continue
		}

		if header[2] != VERSION {
			log.Printf("Invalid version")
			a.handleDisconnect()
			continue
		}

		seq := binary.BigEndian.Uint32(header[4:8])
		ch, ok := a.pendingReqs[seq]
		if !ok {
			log.Printf("Unknown sequence number: %d", seq)
			continue
		}

		bodyLen := binary.BigEndian.Uint32(header[8:12])
		body := make([]byte, bodyLen)
		if _, err := conn.Read(body); err != nil {
			log.Printf("Read body error: %v", err)
			a.handleDisconnect()
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
		case CmdDbs:
			rs, err := command.DeserializeResultDbs(body)
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
			for _, cback := range callback {
				go cback(body)
			}
		}
	}
}

func (a *Ahrikv) Close() error {
	a.running = false
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

func (a *Ahrikv) Dbs() (*command.ResultDbs, error) {
	cmd := command.Dbs{}

	rs, err := a.send(CmdDbs, cmd)
	if err != nil {
		return nil, err
	}
	msg := <-rs
	if msg.err != nil {
		return nil, msg.err
	}
	return msg.value.(*command.ResultDbs), nil
}
