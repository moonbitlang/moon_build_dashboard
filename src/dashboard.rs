use serde::{Deserialize, Serialize};

use crate::util::MooncakeSource;

pub type MoonVersion = String;
pub type MooncVersion = String;

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolChainVersion {
    moon_version: MoonVersion,
    moonc_version: MooncVersion,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MoonBuildDashboard {
    stable_release: ToolChainVersion,
    nightly_release: ToolChainVersion,
    data: Vec<MooncakeState>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MooncakeState {
    source: MooncakeSource,
    results: BuildState,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    Success,
    Failure,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteResult {
    status: Status,
    elapsed: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BuildState {
    check: ExecuteResult,
    build: ExecuteResult,
    test: ExecuteResult,
}
