import { Client, QueryResult } from "pg";
import { getRestaurantDetails } from "./scraper";
import { Coords, RestaurantDetails } from "./types";
import * as dotenv from "dotenv";
import { readFile } from "fs";
dotenv.config();

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

export interface Restaurant {
  id_restaurant: number;
  url: string;
  name: string;
  coords?: Coords;
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
  for (let index = 0; index < MONTHS.length; index++) {
    month = MONTHS[index];
    if (date.includes(month)) {
      const day = date.substring(0, date.indexOf(month) - 1);
      const monthNumber = index + 1;
      const year = date.substring(
        date.indexOf(month) + month.length + 1,
        date.length
      );
      return `${year}-${monthNumber}-${day.substring(
        day.length - 2,
        day.length
      )}`;
    }
  }
  return "";
}

export async function createTables(): Promise<void> {
  const client = new Client(clientInfo);
  console.log(`Connecting to database ${clientInfo.database} on host ${clientInfo.host}`);
  await client.connect();
  console.log("Connected to database");
  readFile("scripts/create_tables.sql", "utf8", (err, data) => {
    if (err) {
      console.error(err);
      return;
    }
    console.log(data)
    client.query(data).then(() => {
      console.log("Tables created");
      client.end();
    }).catch((err) => {
      console.error(err);
      client.end();
    });
    
    return;
  }
  );
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
  return restaurants;
}

async function insertRestaurantCategories(client: Client) {
  const existingCategories = await client.query(
    "SELECT * FROM cat_suggestions"
  );
  if (existingCategories.rowCount > 0) {
    return;
  }

  for (const category of [
    Keyword_Category.RESTAURANT,
    Keyword_Category.FOOD,
    Keyword_Category.PERIOD,
  ]) {
    await client.query(
      "INSERT INTO cat_suggestions (idcat, namecat) VALUES ($1, $2)",
      [category, Keyword_Category[category]]
    );
  }
}

async function insertMealIntoDB(menu: MealSql, client: Client) {
  await client.query(
    "INSERT INTO meal (typemeal, foodies, day, idrestaurant) VALUES ($1, $2, $3, $4)",
    [menu.type, menu.foodies, menu.day, menu.id_restaurant]
  );
}

export async function insertRestaurantsInDB(restaurants: Restaurant[]) {
  const client = new Client(clientInfo);
  await client.connect();
  const sqlPromises: Promise<
    QueryResult<{
      idrestaurant: string;
      url: string;
      name: string;
    }>
  >[] = [];
  restaurants.forEach((restaurant) => {
    const query = `INSERT INTO restaurant(url, name${
      restaurant.coords ? ", gpscoord" : ""
    }) VALUES($1,$2${restaurant.coords ? ",$3" : ""})`;
    const values = [restaurant.url, restaurant.name];
    if (restaurant.coords) {
      values.push(`(${restaurant.coords.x},${restaurant.coords.y})`);
    }
    sqlPromises.push(client.query(query, values));
  });
  await Promise.all(sqlPromises);
  await client.end();
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

export async function clearRestaurantTable() {
  const client = new Client(clientInfo);
  await client.connect();
  await client.query("DELETE FROM restaurant");
  client.end();
}

export async function updateMeals() {
  await clearMealAndSuggestionTables();
  const restaurants = await getRestaurants();
  const keyword = new Map<string, Keyword[]>(); // reversed index table for search system

  let restaurant_details: RestaurantDetails | null = null;
  const client = new Client(clientInfo);
  await client.connect();
  await insertRestaurantCategories(client);
  for (const restaurant of restaurants) {
    const restaurant_keyword: Keyword = {
      category: Keyword_Category.RESTAURANT,
      id_entity: restaurant.id_restaurant,
    };
    keyword.set(restaurant.name, [restaurant_keyword]);
    try {
      restaurant_details = await getRestaurantDetails(restaurant.url);
    } catch (e) {
      console.error(e);
    }

    if (restaurant_details === null) {
      continue;
    }

    for (const menu of restaurant_details.food_page.menus) {
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
  const query =
    "INSERT INTO suggestions_restaurant(keyword, idRestaurant, idcat)  VALUES($1,$2,$3)";
  const sqlPromises: Promise<
    QueryResult<{
      keyword: string;
      idrestaurant: number;
      idcat: number;
    }>
  >[] = [];
  keyword.forEach((val, key) => {
    for (const keyword_conf of val || [{ category: "", id_entity: 0 }]) {
      try {
        sqlPromises.push(
          client.query(query, [
            key,
            keyword_conf.id_entity,
            keyword_conf.category,
          ])
        );
      } catch (e) {
        console.error(e);
      }
    }
  });
  await Promise.all(sqlPromises);
  await client.end();
}

// async function insertRestaurantsUpdate() {
//   await clearRestaurantTable();
//   const restaurants = await insertRestaurants()
//   await insertRestaurantsInDB(restaurants);
// }
// insertRestaurantsUpdate()
//
//
// console.time("took");
// updateMeals().then(()=> {
//   console.timeEnd("took");
