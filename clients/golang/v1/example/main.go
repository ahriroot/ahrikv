package main

import (
	"fmt"

	v1 "github.com/ahriroot/ahrikv/clients/golang/v1"
)

func main() {
	akv, err := v1.NewAhrikv(v1.Config{
		Host:   "43.139.86.190",
		Port:   60002,
		Secret: "QDC2$/}Mx-Knf+//!K%{<V1D",
	}, "auth")
	if err != nil {
		println("connect to akv server failed")
		panic(err)
	}

	keys, err := akv.Keys("*")
	if err != nil {
		panic(err)
	}

	fmt.Println("Keys:", keys)
}
