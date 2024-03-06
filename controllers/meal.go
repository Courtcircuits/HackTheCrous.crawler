package controllers

import (
	"database/sql"
	"fmt"

	"github.com/PuerkitoBio/goquery"
)

type Meal struct {
	Typemeal     string `json:"typemeal"`
	Foodies      string `json:"foodies"`
	Day          string `json:"day"`
	Idrestaurant int    `json:"idrestaurant"`
}

type Foody struct {
	Place  string   `json:"place"`
	Values []string `json:"values"`
}

type SqlMeal struct {
	Idmeal       sql.NullInt32  `json:"idmeal"`
	Typemeal     sql.NullString `json:"typemeal"`
	Foodies      sql.NullString `json:"foodies"`
	Day          sql.NullTime   `json:"day"`
	Idrestaurant sql.NullInt32  `json:"idrestaurant"`
}

type IMealController interface {
	GetAll() ([]Meal, error)
	Scrape(url string) error
}

type MealController struct{}

func (m *Meal) Save() error {
	client, err := Db.Connect()
	if err != nil {
		return err
	}
	defer client.Close()
	query := `INSERT INTO meals (typemeal, foodies, day, idrestaurant) VALUES ($1, $2, $3, $4)`

	_, err = client.Exec(query, m.Typemeal, m.Foodies, m.Day, m.Idrestaurant)
	return err
}

func (sm *SqlMeal) ToMeal() Meal {
	return Meal{
		Typemeal:     sm.Typemeal.String,
		Foodies:      sm.Foodies.String,
		Day:          sm.Day.Time.String(),
		Idrestaurant: int(sm.Idrestaurant.Int32),
	}
}

func (mc *MealController) GetAll() ([]Meal, error) {
	client, err := Db.Connect()
	if err != nil {
		return nil, err
	}
	defer client.Close()
	query := `SELECT * FROM meals`
	rows, err := client.Query(query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()
	var meals []Meal
	for rows.Next() {
		var sm SqlMeal
		err := rows.Scan(&sm.Idmeal, &sm.Typemeal, &sm.Foodies, &sm.Day, &sm.Idrestaurant)
		if err != nil {
			return nil, err
		}
		meals = append(meals, sm.ToMeal())
	}
	return meals, nil
}

func ScrapeMeal(restaurant_url string) ([]Meal, error) {
	html, err := requestHtml(restaurant_url)

	doc, err := goquery.NewDocumentFromReader(html)
	if err != nil {
		return nil, fmt.Errorf("goquery not able to parse html: %v", err)
	}

	meals := []Meal{}

	doc.Find(".meal").Each(func(i int, s *goquery.Selection) {
		meal := Meal{}
		s.Find(".meal_title").Each(func(i int, s *goquery.Selection) {
			meal.Typemeal = s.Text()
		})
		s.Find("ul.meal_foodies").Each(func(i int, s *goquery.Selection) {

		})
	})

}
