package main

import (
	"errors"
	"fmt"
	"time"

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

	err = akv.Connect()
	if err != nil {
		println("connect to akv server failed")
		panic(err)
	}

	result, err := akv.Set("key", "value", 5)
	if err != nil {
		panic(err)
	}

	fmt.Println("Set result:", result)

	keys, err := akv.Keys("*")
	if err != nil {
		panic(err)
	}

	fmt.Println("Keys:", keys)

	exists, err := akv.Exists("key")
	if err != nil {
		panic(err)
	}

	fmt.Println("Exists:", exists)

	ds, err := akv.Del("key")
	if err != nil {
		if errors.Is(err, v1.AkvNil) {
			fmt.Println("key not found")
		} else {
			panic(err)
		}
	}
	fmt.Println("Del result:", ds)

	exists, err = akv.Exists("key")
	if err != nil {
		panic(err)
	}

	fmt.Println("Exists:", exists)

	keys, err = akv.Keys("*")
	if err != nil {
		panic(err)
	}

	fmt.Println("Keys:", keys)

	// sleep for 6 seconds
	time.Sleep(time.Second * 6)

	rs, err := akv.Get("key")
	if err != nil {
		if errors.Is(err, v1.AkvNil) {
			fmt.Println("key not found")
		} else {
			panic(err)
		}
	}

	fmt.Println("Get result:", rs)

	result, err = akv.Set("key", "value", 5)
	if err != nil {
		panic(err)
	}
	rs, err = akv.Get("key")
	if err != nil {
		if errors.Is(err, v1.AkvNil) {
			fmt.Println("key not found")
		} else {
			panic(err)
		}
	}
	var exp uint64 = 10
	isExpire, err := akv.Expire("key", &exp)
	if err != nil {
		panic(err)
	}

	fmt.Println("Expire result:", isExpire)

	rs, err = akv.Get("key")
	if err != nil {
		if errors.Is(err, v1.AkvNil) {
			fmt.Println("key not found")
		} else {
			panic(err)
		}
	}
	fmt.Printf("Get result: %+v\n", rs)
}
