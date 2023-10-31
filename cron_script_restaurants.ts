import { Restaurant } from "./cron_script";
import { getRestaurantUrls } from "./scraper";

export async function insertRestaurants(): Promise<Restaurant[]>{
  const url = "https://www.crous-montpellier.fr/se-restaurer/ou-manger/";
  const restaurants = await getRestaurantUrls(url);
  return restaurants;
}

