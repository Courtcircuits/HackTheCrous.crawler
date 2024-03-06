package controllers

import (
	"database/sql"
	"fmt"

	"github.com/Courtcircuits/HackTheCrous.crawler/utils"
)

type Database struct {
	conn_string string
}

var Db *Database

func NewDatabase() *Database {
	conn_string := utils.GetDBURL()
	return &Database{
		conn_string: conn_string,
	}
}

func (d *Database) Connect() (*sql.DB, error) {
	fmt.Println("Connecting to", d.conn_string)
	db, err := sql.Open("postgres", d.conn_string)
	if err != nil {
		return nil, err
	}
	return db, nil
}
