use std::fmt::Display;
use std::process::ExitCode;
use std::sync::Arc;

use async_trait::async_trait;
use serde_derive::Deserialize;
use serde_derive::Serialize;

use crate::cli::Action;
use crate::cli::ExitResult;
use crate::models::schools::School;
use crate::models::schools::SchoolService;

// Documentation here : https://www.herault-data.fr/explore/dataset/onisep-etablissements-denseignement-superieur-herault/api/

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeraultData {
    #[serde(rename = "total_count")]
    pub total_count: i64,
    pub results: Vec<ApiSchool>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiSchool {
    #[serde(rename = "code_uai")]
    pub code_uai: String,
    #[serde(rename = "ndeg_siret")]
    pub ndeg_siret: Option<f64>,
    #[serde(rename = "type_d_etablissement")]
    pub type_d_etablissement: String,
    pub nom: String,
    pub sigle: Option<String>,
    pub statut: String,
    pub tutelle: Option<String>,
    pub universite: Option<String>,
    #[serde(rename = "boite_postale")]
    pub boite_postale: Option<String>,
    pub adresse: String,
    pub cp: f64,
    pub commune: String,
    pub telephone: String,
    #[serde(rename = "debut_portes_ouvertes")]
    pub debut_portes_ouvertes: Option<String>,
    #[serde(rename = "fin_portes_ouvertes")]
    pub fin_portes_ouvertes: Option<String>,
    #[serde(rename = "commentaires_portes_ouvertes")]
    pub commentaires_portes_ouvertes: Option<String>,
    #[serde(rename = "lien_site_onisep_fr")]
    pub lien_site_onisep_fr: String,
    #[serde(rename = "point_geo")]
    pub point_geo: PointGeo,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PointGeo {
    pub lon: f64,
    pub lat: f64,
}

impl Display for PointGeo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{})", self.lat, self.lon)
    }
}

pub struct SchoolAction {
    pub school_service: Arc<SchoolService>,
}

impl SchoolAction {
    pub fn new(school_service: Arc<SchoolService>) -> Self {
        Self { school_service }
    }
}

#[async_trait]
impl Action for SchoolAction {
    async fn execute(&self) -> Result<ExitResult, ExitResult> {
        const BASE_URL: &str = "https://www.herault-data.fr/api/explore/v2.1/catalog/datasets/onisep-etablissements-denseignement-superieur-herault/records";

        // Fetch both pages concurrently
        let (url_page1, url_page2) = (
            format!("{}?limit=100", BASE_URL),
            format!("{}?limit=100&offset=100", BASE_URL),
        );
        let (page1, page2) = tokio::try_join!(
            fetch_schools_page(&url_page1),
            fetch_schools_page(&url_page2)
        )?;

        // Process both pages
        let schools: Vec<ApiSchool> = filter_public_schools(page1.results)
            .into_iter()
            .chain(filter_public_schools(page2.results))
            .collect();

        // Clear existing schools
        self.school_service.clear().await.map_err(|e| ExitResult {
            exit_code: ExitCode::FAILURE,
            message: format!("school clear failed: {}", e),
        })?;

        // Insert new schools
        for school_data in schools {
            let school = convert_to_school(school_data);
            self.school_service
                .create(school)
                .await
                .map_err(|e| ExitResult {
                    exit_code: ExitCode::FAILURE,
                    message: format!("school insertion failed: {}", e),
                })?;
        }

        Ok(ExitResult {
            exit_code: ExitCode::SUCCESS,
            message: "schools inserted".to_string(),
        })
    }

    fn help(&self) -> &str {
        "scrape schools from the given school"
    }
}

async fn fetch_schools_page(url: &str) -> Result<HeraultData, ExitResult> {
    let response = reqwest::get(url).await.map_err(|e| ExitResult {
        exit_code: ExitCode::FAILURE,
        message: format!("reqwest error: {}", e),
    })?;

    let data = response.text().await.map_err(|e| ExitResult {
        exit_code: ExitCode::FAILURE,
        message: format!("reqwest error: {}", e),
    })?;

    serde_json::from_str(&data).map_err(|e| ExitResult {
        exit_code: ExitCode::FAILURE,
        message: format!("serde_json error: {}", e),
    })
}

fn filter_public_schools(schools: Vec<ApiSchool>) -> Vec<ApiSchool> {
    schools
        .into_iter()
        .filter(|school| school.statut.contains("Public"))
        .collect()
}

fn convert_to_school(school_data: ApiSchool) -> School {
    School {
        idschool: 0,
        name: school_data.sigle.unwrap_or(school_data.nom.clone()),
        coords: school_data.point_geo.to_string(),
        long_name: school_data.nom,
    }
}
