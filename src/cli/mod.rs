use std::{collections::HashMap, env, process::ExitCode};

use async_trait::async_trait;

pub struct ExitResult {
    pub exit_code: ExitCode,
    pub message: String,
}

pub mod actions;

#[async_trait]
pub trait Action {
    async fn execute(&self) -> Result<ExitResult, ExitResult>;
}

pub struct Cli {
    actions: HashMap<String, Box<dyn Action>>,
}

impl Cli {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
        }
    }

    pub async fn execute(&mut self) -> Result<ExitResult, ExitResult> {
        let args: Vec<String> = env::args().collect();
        if args.len() > 2 {
            return Err(ExitResult {
                exit_code: ExitCode::from(2),
                message: "too many arguments".to_string(),
            });
        }

        if args.len() < 1 {
            return Err(ExitResult {
                exit_code: ExitCode::from(2),
                message: "missing an argument".to_string(),
            });
        }

        match self.actions.get(args.get(1).unwrap()) {
            Some(command) => command.execute().await,
            None => Err(ExitResult {
                exit_code: ExitCode::from(2),
                message: format!("{} command not found", args[1]),
            }),
        }
    }

    pub fn subscribe_action<T: Action + 'static>(&mut self, caller: &str, action: T) -> &mut Self {
        self.actions.insert(caller.to_string(), Box::new(action));
        self
    }
}
