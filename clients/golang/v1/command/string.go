package command

import "encoding/json"

// ========== SetString ==========
type SetString struct {
	DB     string  `json:"db"`
	Key    string  `json:"key"`
	Value  string  `json:"value"`
	Expire *uint64 `json:"expire"`
}

func (s SetString) Serialize() ([]byte, error) {
	return json.Marshal(s)
}

type ResultSetString struct {
	OK  bool   `json:"ok"`
	Msg string `json:"msg"`
}

func DeserializeResultSetString(data []byte) (*ResultSetString, error) {
	var rs ResultSetString
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}

// ========== GetString ==========
type GetString struct {
	DB  string `json:"db"`
	Key string `json:"key"`
}

func (g GetString) Serialize() ([]byte, error) {
	return json.Marshal(g)
}

type ResultGetString struct {
	Value  *string `json:"value"`
	Expire *uint64 `json:"expire"`
}

func DeserializeResultGetString(data []byte) (*ResultGetString, error) {
	var rs ResultGetString
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}

// ========== DelString ==========
type DelString struct {
	DB  string `json:"db"`
	Key string `json:"key"`
}

func (d DelString) Serialize() ([]byte, error) {
	return json.Marshal(d)
}

type ResultDelString struct {
	Value  *string `json:"value"`
	Expire *uint64 `json:"expire"`
}

func DeserializeResultDelString(data []byte) (*ResultDelString, error) {
	var rs ResultDelString
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}

// ========== ExistsString ==========
type ExistsString struct {
	DB  string `json:"db"`
	Key string `json:"key"`
}

func (e ExistsString) Serialize() ([]byte, error) {
	return json.Marshal(e)
}

type ResultExistsString struct {
	Exists bool `json:"exists"`
}

func DeserializeResultExistsString(data []byte) (*ResultExistsString, error) {
	var rs ResultExistsString
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}

// ========== ExpireString ==========
type ExpireString struct {
	DB     string  `json:"db"`
	Key    string  `json:"key"`
	Expire *uint64 `json:"expire"`
}

func (e ExpireString) Serialize() ([]byte, error) {
	return json.Marshal(e)
}

type ResultExpireString struct {
	OK bool `json:"ok"`
}

func DeserializeResultExpireString(data []byte) (*ResultExpireString, error) {
	var rs ResultExpireString
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}
