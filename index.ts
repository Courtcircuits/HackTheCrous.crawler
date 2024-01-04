import { Command } from "commander";
import { clearRestaurantTable, insertRestaurantsInDB } from "./controller";
import { getRestaurantCoordinates, getRestaurantUrls } from "./scraper";

const program = new Command();

program.version("0.0.1").description("Populate your HackTheCrous database with Crous data");

program.command("restaurants").description("Populate your database with Crous data").action(async ()=> {
  await clearRestaurantTable();
  const url = "https://www.crous-montpellier.fr/se-restaurer/ou-manger/";
  const restaurants = await getRestaurantUrls(url);
  for (const restaurant of restaurants) {
    console.log(`Getting coordinates for ${restaurant.name}...`)
    restaurant.coords = await getRestaurantCoordinates(restaurant.url);
  }
  await insertRestaurantsInDB(restaurants);
  console.log("Your database now contains the Crous restaurants.");
})

program.parse(process.argv);
