package v1

import (
	"fmt"
)

type AkvError struct {
	Code    int
	Message string
}

func (e AkvError) Error() string {
	return fmt.Sprintf("AkvError %d: %s", e.Code, e.Message)
}

var (
	ErrInvalidSecret = AkvError{100, "Invalid secret"}
	AkvNil           = AkvError{200, "Not found"}
)
