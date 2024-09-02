use serde::{Deserialize, Serialize};
use util::{MoonCommand, MooncakeSource};

pub mod cli;
pub mod dashboard;
pub mod git;
pub mod transform;
pub mod util;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Statistics {
    pub source: MooncakeSource,
    pub command: MoonCommand,
    pub moon_version: String,
    pub moonc_version: String,
    pub status: i32,
    pub elapsed: Option<u64>, // None for failed cases
    pub start_time: String,
    pub run_id: String,
    pub run_number: String,
}
