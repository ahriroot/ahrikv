package main

import (
	"fmt"

	v1 "github.com/ahriroot/ahrikv/clients/golang/v1"
)

func main() {
	akv, err := v1.NewAhrikv(v1.Config{
		Host:   "127.0.0.1",
		Port:   60003,
		Secret: "your_secret",
	}, "dbname")
	if err != nil {
		println("connect to akv server failed")
		panic(err)
	}

	err = akv.HashSet("key", "field", "value", 5)
	if err != nil {
		panic(err)
	}

	ex, err := akv.HashExists("key", "field")
	if err != nil {
		panic(err)
	}
	fmt.Println("HashExists result 1:", ex.Exists)

	value, err := akv.HashGet("key", "field")
	if err != nil {
		panic(err)
	}
	fmt.Println("HashGet result:", value)

	len, err := akv.HashLen("key")
	if err != nil {
		panic(err)
	}
	fmt.Println("HashLen result:", len.Len)

	fields, err := akv.HashFields("key")
	if err != nil {
		panic(err)
	}
	fmt.Println("HashKeys result:", fields.Fields)

	r, err := akv.HashDel("key", "field")
	if err != nil {
		panic(err)
	}
	fmt.Println("HashDel result:", r)

	ex, err = akv.HashExists("key", "field")
	if err != nil {
		panic(err)
	}
	fmt.Println("HashExists result 2:", ex.Exists)
}
