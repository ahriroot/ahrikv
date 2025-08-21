package command

import "encoding/json"

// ========== HashSet ==========
type HashSet struct {
	DB     string  `json:"db"`
	Key    string  `json:"key"`
	Field  string  `json:"field"`
	Value  string  `json:"value"`
	Expire *uint64 `json:"expire"`
}

func (h HashSet) Serialize() ([]byte, error) {
	return json.Marshal(h)
}

type ResultHashSet struct {
	OK  bool   `json:"ok"`
	Msg string `json:"msg"`
}

func DeserializeResultHashSet(data []byte) (*ResultHashSet, error) {
	var rs ResultHashSet
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}

// ========== HashGet ==========
type HashGet struct {
	DB    string `json:"db"`
	Key   string `json:"key"`
	Field string `json:"field"`
}

func (h HashGet) Serialize() ([]byte, error) {
	return json.Marshal(h)
}

type ResultHashGet struct {
	Value  *string `json:"value"`
	Expire *uint64 `json:"expire"`
}

func DeserializeResultHashGet(data []byte) (*ResultHashGet, error) {
	var rs ResultHashGet
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}

// ========== HashDel ==========
type HashDel struct {
	DB    string `json:"db"`
	Key   string `json:"key"`
	Field string `json:"field"`
}

func (h HashDel) Serialize() ([]byte, error) {
	return json.Marshal(h)
}

type ResultHashDel struct {
	Value  *string `json:"value"`
	Expire *uint64 `json:"expire"`
}

func DeserializeResultHashDel(data []byte) (*ResultHashDel, error) {
	var rs ResultHashDel
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}

// ========== HashExists ==========
type HashExists struct {
	DB    string `json:"db"`
	Key   string `json:"key"`
	Field string `json:"field"`
}

func (h HashExists) Serialize() ([]byte, error) {
	return json.Marshal(h)
}

type ResultHashExists struct {
	Exists bool `json:"exists"`
}

func DeserializeResultHashExists(data []byte) (*ResultHashExists, error) {
	var rs ResultHashExists
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}

// ========== HashLen ==========
type HashLen struct {
	DB  string `json:"db"`
	Key string `json:"key"`
}

func (h HashLen) Serialize() ([]byte, error) {
	return json.Marshal(h)
}

type ResultHashLen struct {
	Len uint64 `json:"len"`
}

func DeserializeResultHashLen(data []byte) (*ResultHashLen, error) {
	var rs ResultHashLen
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}

// ========== HashKeys ==========
type HashFields struct {
	DB  string `json:"db"`
	Key string `json:"key"`
}

func (h HashFields) Serialize() ([]byte, error) {
	return json.Marshal(h)
}

type ResultHashFields struct {
	Fields []string `json:"fields"`
	Total  uint32   `json:"total"`
}

func DeserializeResultHashFields(data []byte) (*ResultHashFields, error) {
	var rs ResultHashFields
	err := json.Unmarshal(data, &rs)
	if err != nil {
		return nil, err
	}
	return &rs, nil
}
