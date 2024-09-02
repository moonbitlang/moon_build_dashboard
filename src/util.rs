use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MooncakeSource {
    MooncakesIO { name: String, version: String },
    Git { url: String, rev: String },
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

pub fn moon_version() -> anyhow::Result<String> {
    let output = std::process::Command::new("moon")
        .args(["version"])
        .output()?;
    let version = String::from_utf8(output.stdout)?;
    Ok(version.trim().to_string())
}

pub fn moonc_version() -> anyhow::Result<String> {
    let output = std::process::Command::new("moonc").args(["-v"]).output()?;
    let version = String::from_utf8(output.stdout)?;
    Ok(version.trim().to_string())
}
