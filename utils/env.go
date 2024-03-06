package utils

import (
	"fmt"
	"log"
	"os"

	"github.com/spf13/viper"
)

type Config struct {
	Host     string `json:"host,omitempty"`
	Password string `json:"password,omitempty"`
	Username string `json:"username,omitempty"`
	Database string `json:"database,omitempty"`
	Port     string `json:"port,omitempty"`
}

func get(key string) string {
	viper.SetConfigFile("./.env")
	err := viper.ReadInConfig()

	if err != nil {
		return os.Getenv(key) //for production
	}

	value, ok := viper.Get(key).(string)

	if !ok {
		log.Printf("Env variable '%q' not found\n", key)
		return ""
	}

	return value
}

func GetConfig() Config {
	return Config{
		Host:     get("PG_DATABASE"),
		Password: get("PG_PASSWORD"),
		Username: get("PG_USER"),
		Database: get("PG_DATABASE"),
		Port:     get("PG_PORT"),
	}
}

func GetDBURL() string {
	config := GetConfig()
	return fmt.Sprintf("user=%s password=%s host=%s port=%s dbname=%s sslmode=disable", config.Username, config.Password, config.Host, config.Port, config.Database)
}
