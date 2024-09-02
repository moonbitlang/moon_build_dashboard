use serde::{Deserialize, Serialize};

pub mod git;
pub mod util;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Statistics {
    pub repo: String,
    pub rev: String,
    pub command: MoonCommand,
    pub moon_version: String,
    pub moonc_version: String,
    pub status: i32,
    pub elapsed: Option<u64>, // None for failed cases
    pub start_time: String,
    pub run_id: String,
    pub run_number: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MoonCommand {
    Check,
    Build,
    Test,
    Bundle,
}

impl MoonCommand {
    pub fn args(&self) -> Vec<&str> {
        match self {
            MoonCommand::Check => vec!["check", "-q"],
            MoonCommand::Build => vec!["build", "-q"],
            MoonCommand::Test => vec!["test", "-q", "--build-only"],
            MoonCommand::Bundle => vec!["bundle", "-q"],
        }
    }
}
