import { Client, QueryResult } from "pg";
import { getRestaurantDetails } from "./scraper";
import { RestaurantDetails } from "./types";
import * as dotenv from 'dotenv'
dotenv.config()

enum Keyword_Category {
  RESTAURANT = 1,
  FOOD = 2,
  PERIOD = 3,
}

interface Keyword {
  id_entity: number;
  category: Keyword_Category;
}

interface RestaurantSql {
  idrestaurant: string;
  url: string;
  name: string;
}

interface Restaurant {
  id_restaurant: number;
  url: string;
  name: string;
}

interface MealSql {
  type: string;
  foodies: string;
  day: string;
  id_restaurant: number;
}

const clientInfo = {
  user: process.env.PG_USER || "",
  password: process.env.PG_PASSWORD || "",
  host: process.env.PG_HOST || "",
  port: parseInt(process.env.PG_PORT || "0"),
  database: process.env.PG_DATABASE || "", //microsoft wont get my credentials
};

function formatDate(date: string): string {
  const MONTHS = [
    "janvier",
    "février",
    "mars",
    "avril",
    "mai",
    "juin",
    "juillet",
    "août",
    "septembre",
    "octobre",
    "novembre",
    "décembre",
  ];
  let month;
  for(let index=0; index<MONTHS.length; index++) {
    month = MONTHS[index]
    if (date.includes(month)) {
      const day = date.substring(0, date.indexOf(month) - 1);
      const monthNumber = index + 1;
      const year = date.substring(
        date.indexOf(month) + month.length + 1,
        date.length
      );
      return `${year}-${monthNumber}-${day.substring(day.length-2, day.length)}`;
    }
  }
  return "";
}

async function getRestaurants(): Promise<Restaurant[]> {
  const client = new Client(clientInfo);
  await client.connect();
  const result = await client.query(
    "SELECT idrestaurant, url, name FROM restaurant"
  );
  await client.end();

  const restaurants: Restaurant[] = [];

  result.rows.map((restaurant: RestaurantSql) => {
    restaurants.push({
      id_restaurant: parseInt(restaurant.idrestaurant),
      url: restaurant.url,
      name: restaurant.name,
    });
  });
  console.log(restaurants)
  return restaurants;
}

async function insertMealIntoDB(menu: MealSql, client: Client) {
  console.log(menu)
  await client.query(
    "INSERT INTO meal (typemeal, foodies, day, idrestaurant) VALUES ($1, $2, $3, $4)",
    [menu.type, menu.foodies, menu.day, menu.id_restaurant]
  );
}

// clear meal and suggetion_restaurant tables by deleting all the tupples to make sure no old data is kept
async function clearMealAndSuggestionTables() {
  const client = new Client(clientInfo);
  await client.connect();
  const queries = [
    client.query("DELETE FROM meal"),
    client.query("DELETE FROM suggestions_restaurant"),
  ];
  await Promise.all(queries);
  client.end();
}

async function updateMeals() {
  await clearMealAndSuggestionTables();
  const restaurants = await getRestaurants();
  const keyword = new Map<string, Keyword[]>(); // reversed index table for search system

  let restaurant_details : RestaurantDetails | null = null;
  const client = new Client(clientInfo);
  await client.connect();
  for (const restaurant of restaurants) {
    const restaurant_keyword: Keyword = {
      category: Keyword_Category.RESTAURANT,
      id_entity: restaurant.id_restaurant,
    };
    keyword.set(restaurant.name, [restaurant_keyword]);
    try{
      restaurant_details = await getRestaurantDetails(restaurant.url);
    }catch(e){
      console.error((e))
    }

    if (restaurant_details === null) {
      continue;
    }

    

    for (const menu of restaurant_details.food_page.menus) {
      console.log(menu)
      if (!keyword.has(menu.title)) {
        keyword.set(menu.title, []);
      }
      keyword.get(menu.title)?.push({
        category: Keyword_Category.RESTAURANT,
        id_entity: restaurant.id_restaurant,
      });

      for (const foody of menu.foodies) {
        for (const food of foody.content) {
          if (!keyword.has(food)) {
            keyword.set(food, []);
          }
          keyword.get(food)?.push({
            id_entity: restaurant.id_restaurant,
            category: Keyword_Category.FOOD,
          });
        }
      }

      await insertMealIntoDB(
        {
          day: formatDate(restaurant_details.food_page.time),
          foodies: JSON.stringify(menu.foodies),
          id_restaurant: restaurant.id_restaurant,
          type: menu.title,
        },
        client
      );
    }
  }
  console.log(keyword);

  const query = "INSERT INTO Suggestions_Restaurant(keyword, idRestaurant, idcat)  VALUES($1,$2,$3)";
  const sqlPromises: Promise<QueryResult<any>>[] = [];
  for (const key in keyword.keys){
    console.log(key)
    for (const keyword_conf of keyword.get(key) || [{category:"", id_entity:0}]){
      sqlPromises.push(client.query(query, [key, keyword_conf.id_entity, keyword_conf.category]))
    }
  }
  await Promise.all(sqlPromises)
  await client.end();
}

console.time("took");
updateMeals().then(()=> {
  console.timeEnd("took");
})
