package command

import "encoding/json"

type Command interface {
	Serialize() ([]byte, error)
}

// ========== Ping ==========
type Ping struct {
}

func (s Ping) Serialize() ([]byte, error) {
	return json.Marshal(s)
}

type ResultPing struct {
}

func DeserializeResultPing(data []byte) (*ResultPing, error) {
	var result ResultPing
	err := json.Unmarshal(data, &result)
	if err != nil {
		return nil, err
	}
	return &result, nil
}

// ========== Error ==========

type ResultError struct {
	Cmd uint8  `json:"cmd"`
	Msg string `json:"msg"`
}

func DeserializeResultError(data []byte) (*ResultError, error) {
	var result ResultError
	err := json.Unmarshal(data, &result)
	if err != nil {
		return nil, err
	}
	return &result, nil
}

// ========== Authenticate ==========
type Authenticate struct {
	Secret string `json:"secret"`
}

func (s Authenticate) Serialize() ([]byte, error) {
	return json.Marshal(s)
}

type ResultAuthenticate struct {
	Ok  bool   `json:"ok"`
	Msg string `json:"msg"`
}

func DeserializeResultAuthenticate(data []byte) (*ResultAuthenticate, error) {
	var result ResultAuthenticate
	err := json.Unmarshal(data, &result)
	if err != nil {
		return nil, err
	}
	return &result, nil
}

// ========== Keys ==========
type Keys struct {
	DB   string `json:"db"`
	Page uint   `json:"page"`
	Size uint   `json:"size"`
}

func (s Keys) Serialize() ([]byte, error) {
	return json.Marshal(s)
}

type ResultKeys struct {
	Keys  []string `json:"keys"`
	Total uint32   `json:"total"`
}

func DeserializeResultKeys(data []byte) (*ResultKeys, error) {
	var result ResultKeys
	err := json.Unmarshal(data, &result)
	if err != nil {
		return nil, err
	}
	return &result, nil
}
