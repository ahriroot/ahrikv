package main

import (
	"fmt"
	"time"

	v1 "github.com/ahriroot/ahrikv/clients/golang/v1"
)

func main() {
	akv, err := v1.NewAhrikv(v1.Config{
		Host:   "127.0.0.1",
		Port:   60002,
		Secret: "your_secret",
	})
	if err != nil {
		panic(err)
	}

	result, err := akv.Set("key", "value", 3)
	if err != nil {
		panic(err)
	}

	fmt.Println(result)

	// sleep for 3 seconds
	time.Sleep(time.Second * 3)

	value, expire, err := akv.Get("key")
	if err != nil {
		panic(err)
	}

	fmt.Println(value, expire)
}
