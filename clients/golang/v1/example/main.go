package main

import (
	"fmt"
	"log"
	"time"

	v1 "github.com/ahriroot/ahrikv/clients/golang/v1"
)

func main() {
	config := v1.NewConfig()
	config.Host = "127.0.0.1"
	config.Port = 60002
	config.Secret = "your_secret"
	config.PingInterval = 5 * time.Second
	config.ReconnectInterval = 2 * time.Second
	config.MaxReconnectAttempts = 10

	client, err := v1.NewAhrikv(*config, "default")
	if err != nil {
		log.Fatalf("Failed to create client: %v", err)
	}

	err = client.Connect(func(message interface{}) {
		log.Printf("Received message: %v", message)
	})
	if err != nil {
		log.Fatalf("Failed to connect: %v", err)
	}

	log.Println("Connected to server")

	go func() {
		for i := 0; i < 3; i++ {
			rs, err := client.Set(fmt.Sprintf("test_key_%d", i), fmt.Sprintf("test_value_%d", i))
			if err != nil {
				log.Printf("Failed to set key: %v", err)
				continue
			}
			log.Printf("Set result: %+v", rs)
		}

		for i := 0; i < 3; i++ {
			rs, err := client.Get(fmt.Sprintf("test_key_%d", i))
			if err != nil {
				log.Printf("Failed to get key: %v", err)
				continue
			}
			log.Printf("Get result: %+v", rs)
		}
	}()

	time.Sleep(10 * time.Second)

	log.Println("Closing client...")
	client.Close()
	log.Println("Client closed")
}
