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

export interface School {
  name: string;
  long_name: string;
  coords: Coords;
}

export interface ApiResponse<T> {
  total_count: number;
  results: T[];
}

export interface SchoolApi {
  code_uai: string;
  ndeg_siret?: number;
  type_d_etablissement: string;
  nom: string;
  sigle?: string;
  statut: string;
  tutelle?: string;
  universite?: string;
  boite_postale?: string;
  adresse: string;
  cp: number;
  commune: string;
  telephone: string;
  debut_portes_ouvertes?: string;
  fin_portes_ouvertes?: string;
  commentaires_portes_ouvertes?: string;
  lien_site_onisep_fr: string;
  point_geo: PointGeo;
}

export interface PointGeo {
  lon: number;
  lat: number;
}
