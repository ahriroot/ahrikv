package main

import (
	"log"

	v1 "github.com/ahriroot/ahrikv/clients/golang/v1"
)

func main() {
	akv, err := v1.NewAhrikv(v1.Config{
		Host:   "127.0.0.1",
		Port:   60002,
		Secret: "your_secret",
	}, "default")
	if err != nil {
		println("connect to akv server failed")
		panic(err)
	}

	err = akv.Connect()
	if err != nil {
		println("connect to akv server failed")
		panic(err)
	}

	rs, err := akv.Dbs()
	if err != nil {
		log.Printf("Failed to get databases: %v", err)
		return
	}
	log.Printf("Databases result: %+v", rs)

	keysRs, err := akv.Keys("")
	if err != nil {
		log.Printf("Failed to get keys: %v", err)
		return
	}
	log.Printf("Keys result: %+v", keysRs)
}
