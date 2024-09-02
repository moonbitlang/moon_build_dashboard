use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum MooncakeSource {
    MooncakesIO {
        name: String,
        version: Option<String>,
    },
    Git {
        url: String,
        rev: Option<String>,
    },
}

impl fmt::Display for MooncakeSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MooncakeSource::MooncakesIO { name, version } => {
                write!(
                    f,
                    "{}{}",
                    name,
                    version
                        .as_deref()
                        .map(|v| format!("@{}", v))
                        .unwrap_or("".into())
                )
            }
            MooncakeSource::Git { url, rev } => {
                write!(
                    f,
                    "{}{}",
                    url,
                    rev.as_deref()
                        .map(|v| format!("@{}", v))
                        .unwrap_or("".into())
                )
            }
        }
    }
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolChainVersion {
    pub label: String,
    pub moon_version: String,
    pub moonc_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoonBuildDashboard {
    pub run_id: String,
    pub run_number: String,
    pub start_time: String,

    pub sources: Vec<MooncakeSource>,

    pub stable_toolchain_version: ToolChainVersion,
    pub stable_release_data: Vec<BuildState>,

    pub bleeding_toolchain_version: ToolChainVersion,
    pub bleeding_release_data: Vec<BuildState>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Success,
    Failure,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResult {
    pub status: Status,
    pub start_time: String,
    pub elapsed: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildState {
    pub source: MooncakeSource,
    pub check: ExecuteResult,
    pub build: ExecuteResult,
    pub test: ExecuteResult,
}
