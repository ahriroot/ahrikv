package command_string

type GetString struct {
	DB  string `json:"db"`
	Key string `json:"key"`
}

type InnerResultGetString struct {
	Value  *string `json:"value"`
	Expire *uint64 `json:"expire"`
}

type ResultGetString struct {
	IsNull bool   `json:"is_null"`
	Value  string `json:"value"`
	Expire uint64 `json:"expire"`
}
