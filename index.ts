import { Command } from "commander";
import {
  clearRestaurantTable,
  createTables,
  insertRestaurantsInDB,
  insertSchools,
  updateMeals,
  updateOpenningHours,
} from "./controller";
import {
  getOpenHours,
  getRestaurantCoordinates,
  getRestaurantUrls,
  getSchools,
} from "./scraper";
import * as dotenv from "dotenv";

const program = new Command();
dotenv.config();

program
  .version("0.0.1")
  .description("Populate your HackTheCrous database with Crous data");

program
  .command("restaurants")
  .description("Populate your database with Crous data")
  .action(async () => {
    await clearRestaurantTable();
    const url = "https://www.crous-montpellier.fr/se-restaurer/ou-manger/";
    const restaurants = await getRestaurantUrls(url);
    for (const restaurant of restaurants) {
      console.log(`Getting coordinates for ${restaurant.name}...`);
      restaurant.coords = await getRestaurantCoordinates(restaurant.url);
      restaurant.open_hours = await getOpenHours(restaurant.url);
    }

    await insertRestaurantsInDB(restaurants);
    console.log("Your database now contains the Crous restaurants.");
  });

program
  .command("meals")
  .description("Populate your database with Crous meals")
  .action(() => {
    console.time("took");
    updateMeals().then(() => {
      console.timeEnd("took");
    });
  });

program
  .command("up")
  .description("Import tables into the database")
  .action(async () => {
    console.log(process.env.PG_PASSWORD);
    createTables();
  });

program
  .command("hours")
  .description("Get opening hours of restaurants")
  .action(() => {
    console.time("took");
    updateOpenningHours().then(() => {
      console.timeEnd("took");
    });
  });

program
  .command("school")
  .description("Get schools")
  .action(() => {
    console.time("took");
    getSchools().then((schools) => {
      insertSchools(schools).then(() => {
        console.log(schools);
        console.timeEnd("took");
      });
    });
  });

program.parse(process.argv);
