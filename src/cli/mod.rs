use std::{collections::HashMap, process::ExitCode};

use async_trait::async_trait;
use clap::{Parser, Subcommand};
pub struct ExitResult {
    pub exit_code: ExitCode,
    pub message: String,
}

pub mod actions;

#[async_trait]
pub trait Action {
    async fn execute(&self) -> Result<ExitResult, ExitResult>;
    fn help(&self) -> &str;
}

pub struct Cli {
    actions: HashMap<Command, Box<dyn Action>>,
}

#[derive(Debug, Parser)]
#[clap(name = "htcrawler", version)]
pub struct App{
    #[clap(subcommand)]
    action: Command
}

#[derive(Debug, Subcommand, PartialEq, Eq, Hash)]
pub enum Command {
    Restaurants,
    Meals,
    Up,
    Bootstrap
}

impl Cli {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub async fn execute(&mut self, app: App) -> Result<ExitResult, ExitResult> {
        match self.actions.get(&app.action) {
            Some(command) => {
                command.execute().await
            },
            None => Err(ExitResult {
                exit_code: ExitCode::from(2),
                message: "command not found".to_string(),
            }),
        }
    }
    pub fn subscribe_action<T: Action + 'static>(&mut self, caller: Command, action: T) -> &mut Self {
        self.actions.insert(caller, Box::new(action));
        self
    }
}
