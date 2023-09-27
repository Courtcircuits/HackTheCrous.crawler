export interface MapElement extends Element {
  dataset: {
    lon: string;
    lat: string;
  };
}

export interface Meal {
  title: string;
  foodies: Foody[];
}

export interface Foody {
  type: string;
  content: string[];
}

export interface Food_Page {
  name: string;
  url: string;
  time: string;
  menus: Meal[];
}

export interface RestaurantDetails {
  food_page: Food_Page;
  coords: Coords | null;
}

export interface Coords {
  x: number;
  y: number;
}