package controllers

import (
	"database/sql"
	"fmt"
	"io"
	"log"
	"net/http"
	"strconv"
	"strings"
	"time"

	"github.com/PuerkitoBio/goquery"
)

type Coordinates struct {
	Latitude  float64 `json:"latitude"`
	Longitude float64 `json:"longitude"`
}

func (c Coordinates) String() string {
	return fmt.Sprintf("(%f,%f)", c.Latitude, c.Longitude)
}

type Restaurant struct {
	Url         string      `json:"url"`
	Name        string      `json:"name"`
	Coordinates Coordinates `json:"coordinates"`
}

type SqlRestaurant struct {
	Idrestaurant sql.NullInt32  `json:"idrestaurant"`
	Url          sql.NullString `json:"url"`
	Name         sql.NullString `json:"name"`
	Coordinates  sql.NullString `json:"coordinates"`
}

// for debug
func (sr *SqlRestaurant) ToRestaurant() Restaurant {
	coordinates := strings.Split(sr.Coordinates.String[1:len(sr.Coordinates.String)-1], ",")
	latitude, err := strconv.ParseFloat(coordinates[0], 64)
	if err != nil {
		log.Fatalln(err)
	}
	longitude, err := strconv.ParseFloat(coordinates[1], 64)
	if err != nil {
		log.Fatalln(err)
	}

	return Restaurant{
		Url:  sr.Url.String,
		Name: sr.Name.String,
		Coordinates: Coordinates{
			Latitude:  latitude,
			Longitude: longitude,
		},
	}
}

type IRestaurantController interface {
	// GetAll() []Restaurant
	Scrape(url string) error
	GetAll() ([]Restaurant, error)
}

type RestaurantController struct{}

func (r *Restaurant) Save() error {
	client, err := Db.Connect()
	if err != nil {
		return err
	}
	defer client.Close()
	query := `INSERT INTO restaurants (url, name, coordinates) VALUES ($1, $2, $3)`
	_, err = client.Exec(query, r.Url, r.Name, r.Coordinates)
	return err
}

func (r *RestaurantController) GetAll() ([]Restaurant, error) {
	client, err := Db.Connect()
	if err != nil {
		return nil, err
	}
	defer client.Close()
	query := `SELECT * FROM restaurants`
	rows, err := client.Query(query)
	if err != nil {
		return nil, err
	}
	defer rows.Close()

	restaurants := []Restaurant{}
	for rows.Next() {
		var sr SqlRestaurant
		err = rows.Scan(&sr.Idrestaurant, &sr.Url, &sr.Name, &sr.Coordinates)
		if err != nil {
			return nil, err
		}
		restaurants = append(restaurants, sr.ToRestaurant())
	}
	return restaurants, nil
}

func requestHtml(url string) (io.Reader, error) {
	httpClient := &http.Client{
		Timeout: 10 * time.Second,
	}
	resp, err := httpClient.Get(url)
	if err != nil {
		return nil, err
	}
	if resp.StatusCode != http.StatusOK {
		return nil, fmt.Errorf("status code error: %d %s", resp.StatusCode, resp.Status)
	}
	// defer resp.Body.Close()

	return resp.Body, nil
}

func scrapeRestaurantCoords(ch *chan Restaurant, url string) {
	restaurant_html, err := requestHtml(url)

	restaurant := Restaurant{
		Url: url,
	}
	if err != nil {
		*ch <- restaurant
		return
	}
	doc_restaurant, err := goquery.NewDocumentFromReader(restaurant_html)
	if err != nil {
		*ch <- restaurant
		return
	}

	doc_restaurant.Find("#map").Each(func(i int, s *goquery.Selection) {
		latitude, exist := s.Attr("data-lat")
		if exist {
			restaurant.Coordinates.Latitude, _ = strconv.ParseFloat(latitude, 64)
		}
		longitude, exist := s.Attr("data-lon")
		if exist {
			restaurant.Coordinates.Longitude, _ = strconv.ParseFloat(longitude, 64)
		}
		*ch <- restaurant
	})

}

func ScrapeRestaurant(url string) ([]Restaurant, error) {
	// find_restaurants(root_node)
	html, err := requestHtml(url)
	if err != nil {
		return nil, err
	}

	doc, err := goquery.NewDocumentFromReader(html)
	if err != nil {
		return nil, fmt.Errorf("error loading with goquery http response body. %v", err)
	}

	restaurants := []Restaurant{}

	doc.Find("section.vc_restaurants ul li a").Each(func(i int, s *goquery.Selection) {
		// For each item found, get the band and title
		href, _ := s.Attr("href")
		var area string
		var name string
		s.Find("div.restaurant_title").Each(func(i int, s *goquery.Selection) {
			name = s.Text()
		})
		s.Find("span.restaurant_area").Each(func(i int, s *goquery.Selection) {
			area = s.Text()
		})

		if area == "Montpellier" && name != "" {
			restaurants = append(restaurants, Restaurant{
				Name: name, // belek it could get a dereferencing nil pointer but the name != nil condition is already checked
				Url:  href,
				Coordinates: Coordinates{
					Latitude:  0,
					Longitude: 0,
				},
			})
		}
	})

	ch_coordinates := make(chan Restaurant, len(restaurants))
	for _, restaurant := range restaurants {
		go scrapeRestaurantCoords(&ch_coordinates, restaurant.Url)
	}

	nb_results := 0

	for coords := range ch_coordinates {
		for i, restaurant := range restaurants {
			if coords.Url == restaurant.Url {
				restaurant.Coordinates = coords.Coordinates
				restaurants[i] = restaurant
				nb_results++
				break
			}
		}
		if nb_results == cap(ch_coordinates) {
			close(ch_coordinates)
		}
	}

	return restaurants, nil
}

func NewRestaurantController() *RestaurantController {
	return &RestaurantController{}
}
