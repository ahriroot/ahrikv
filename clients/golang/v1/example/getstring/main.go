package main

import (
	"fmt"

	v1 "github.com/ahriroot/ahrikv/clients/golang/v1"
)

func main() {
	akv, err := v1.NewAhrikv(v1.Config{
		Host:   "127.0.0.1",
		Port:   60002,
		Secret: "your_secret",
	}, "dbname")
	if err != nil {
		println("connect to akv server failed")
		panic(err)
	}

	result, err := akv.Get("key")
	if err != nil {
		panic(err)
	}

	fmt.Println(result)
}
