import { JSDOM } from "jsdom";
import { Foody, MapElement, Meal, RestaurantDetails } from "./types";
import { Restaurant } from "./cron_script";

const DAYS = [
  "lundi",
  "mardi",
  "mercredi",
  "jeudi",
  "vendredi",
  "samedi",
  "dimanche",
];

export async function getRestaurantUrls(url: string): Promise<Restaurant[]> {
  const restaurants: Restaurant[] = [];
  const dom = await JSDOM.fromURL(url);
  const { document } = dom.window;
  const restaurant_elements = document.querySelectorAll(".vc_restaurants ul li a");
  for (const restaurant_element of restaurant_elements) {
    const city = restaurant_element.querySelector(".restaurant_area")?.textContent;
    if (city !== "Montpellier" && city !== "Sète") {
      continue;
    }
    const restaurant_url = restaurant_element.getAttribute("href");
    const restaurant_name = restaurant_element.querySelector(".restaurant_title")?.textContent;

    if (restaurant_url) {
      restaurants.push({
        id_restaurant: 0,
        name: restaurant_name || "",
        url: restaurant_url,
      });
    }
  }
  return restaurants;
}

export async function getRestaurantDetails(
  url: string
): Promise<RestaurantDetails | null> {
  const restaurant_details: RestaurantDetails = {
    coords: null,
    food_page: {
      name: "",
      url: "",
      time: "",
      menus: [],
    },
  };

  const dom = await JSDOM.fromURL(url);
  console.log(dom.window.document.querySelector("h1")?.textContent);
  const { document } = dom.window;
  try {
    const menu_element = document.querySelector(".menu");
    if (menu_element === null) {
      throw new Error(".menu was not found on page : " + url);
    }
    const date_value =
      menu_element?.querySelector(".menu_date_title")?.innerHTML;
    if (date_value === undefined) {
      throw new Error(".menu_date_title was not found on page : " + url);
    }
    const meal_elements = menu_element?.querySelectorAll(".meal");

    const map_element = document.querySelector<MapElement>("#map");

    if (map_element === null) {
      throw new Error("#map was not found on page : " + url);
    }

    for (const day of DAYS) {
      if (date_value.includes(day)) {
        restaurant_details.food_page.time = date_value.substring(
          date_value.indexOf(day) + day.length + 1,
          date_value.length
        );
      }
    }

    const food_name = document.querySelector("h1")?.textContent;
    if (!food_name) {
      throw new Error("h1 was not found on page : " + url);
    }
    restaurant_details.food_page.name = food_name;
    restaurant_details.coords = {
      x: parseFloat(map_element.dataset.lat),
      y: parseFloat(map_element.dataset.lon),
    };

    for (const meal_element of meal_elements) {
      const meal_title = meal_element.querySelector(".meal_title");
      if (meal_title === null) {
        throw new Error(".meal_title not found on page : " + url);
      }
      const meal_data: Meal = {
        title: meal_title.innerHTML,
        foodies: [],
      };

      const foody_elements =
        meal_element.querySelectorAll(".meal_foodies > li");
      let sumOfMealLengths: number;

      for (const foody_element of foody_elements) {
        const foody: Foody = {
          content: [],
          type: "",
        };
        const foods = foody_element.querySelectorAll("ul li");
        sumOfMealLengths = 0;
        for (const food of foods) {
          foody.content.push(food.innerHTML);
          sumOfMealLengths += food.textContent?.length || 0;
        }
        foody.type = foody_element.textContent?.substring(0, foody_element.textContent.length- sumOfMealLengths) || ""
        meal_data.foodies.push(foody);
      }
      restaurant_details.food_page.menus.push(meal_data);
    }
  } catch (e) {
    console.error(e);
    return null;
  }
  return restaurant_details;
}
