use std::process::ExitCode;

use async_trait::async_trait;

use crate::cli::{Action, ExitResult};

pub struct PingAction {
    pub url: String
}

impl PingAction {
    pub fn new(
        url: &str
    ) -> Self {
        Self {
            url: url.to_string()
        }
    }
}

#[async_trait]
impl Action for PingAction {
    async fn execute(&self) -> Result<ExitResult, ExitResult> {
        match reqwest::get(self.url.clone()).await.map_err(|e|{
            ExitResult{
                message: format!("{} is unreachable : {:?}", self.url, e),
                exit_code: ExitCode::FAILURE
            }
        })?.text().await {
            Ok(_) => {
                Ok(ExitResult {
                    message: format!("{} is reachable", self.url),
                    exit_code: ExitCode::SUCCESS
                })   
            }
            Err(err) => {
                Err(
            ExitResult{
                message: format!("{} is unreachable : {:?}", self.url, err),
                exit_code: ExitCode::FAILURE
            })
            }
        }
    }
    fn help(&self) -> &str {
       "sends a request to an url to make sure that the website is reachable"
    }
}
