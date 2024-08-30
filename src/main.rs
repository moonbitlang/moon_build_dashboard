use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use argh::FromArgs;
use chrono::{FixedOffset, Local};
use serde::{Deserialize, Serialize};

#[derive(FromArgs)]
#[argh(description = "...")]
pub struct Stat {
    #[argh(option, description = "specify a repo")]
    repo_url: Option<String>,

    #[argh(option, short = 'f', description = "read repos from file")]
    file: Option<PathBuf>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Statistics {
    repo: String,
    rev: String,
    command: MoonCommand,
    moon_version: String,
    moonc_version: String,
    status: i32,
    elapsed: Option<u64>, // None for failed cases
    start_time: String,
    run_id: String,
    run_number: String,
}

fn run_moon(workdir: &Path, args: &[&str]) -> anyhow::Result<Duration> {
    let start = Instant::now();
    let mut cmd = std::process::Command::new("moon")
        .current_dir(workdir)
        .args(args)
        .spawn()?;
    let exit = cmd.wait()?;
    if !exit.success() {
        return Err(anyhow::anyhow!("failed to execute"));
    }
    let elapsed = start.elapsed();
    println!(
        "moon {}, elapsed: {}ms",
        args.join(" "),
        elapsed.as_millis()
    );
    Ok(elapsed)
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum MoonCommand {
    Check,
    Build,
    Test,
    Bundle,
}

impl MoonCommand {
    fn args(&self) -> Vec<&str> {
        match self {
            MoonCommand::Check => vec!["check", "-q"],
            MoonCommand::Build => vec!["build", "-q"],
            MoonCommand::Test => vec!["test", "-q", "--build-only"],
            MoonCommand::Bundle => vec!["bundle", "-q"],
        }
    }
}

fn stat_moon(
    workdir: &Path,
    repo: &str,
    rev: &str,
    moon_version: &str,
    moonc_version: &str,
    cmd: MoonCommand,
) -> anyhow::Result<Vec<Statistics>> {
    let mut ss = vec![];
    let mut durations: Vec<Option<Duration>> = vec![];
    for _ in 0..2 {
        let _ = run_moon(workdir, &["clean"]);
        let r = run_moon(workdir, &cmd.args());
        let status = r.is_err() as i32;
        let d = r.ok();
        durations.push(d);
        let start_time = Local::now()
            .with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())
            .format("%Y-%m-%d %H:%M:%S.%3f")
            .to_string();
        let elapsed = d.map(|d| d.as_millis() as u64);
        let run_id = std::env::var("GITHUB_ACTION_RUN_ID").unwrap_or("0".into());
        let run_number = std::env::var("GITHUB_ACTION_RUN_NUMBER").unwrap_or("0".into());
        let stat = Statistics {
            repo: repo.to_string(),
            rev: rev.to_string(),
            command: cmd,
            moon_version: moon_version.to_string(),
            moonc_version: moonc_version.to_string(),
            status,
            elapsed,
            start_time,
            run_id,
            run_number,
        };
        ss.push(stat);
    }
    let last = ss.last().unwrap().clone();
    Ok(vec![last])
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

pub fn get_branch_name(workdir: &Path) -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .current_dir(workdir)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()?;
    let branch_name = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(branch_name)
}

pub fn get_git_short_hash(workdir: &Path) -> anyhow::Result<String> {
    let output = std::process::Command::new("git")
        .current_dir(workdir)
        .args(["rev-parse", "--short", "HEAD"])
        .output()?;
    let hash = String::from_utf8(output.stdout)?.trim().to_string();
    Ok(hash)
}

pub fn git_clone_to(repo: &str, workdir: &Path, dst: &str) -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new("git")
        .current_dir(workdir)
        .args(["clone", repo, dst, "--depth", "1"])
        .spawn()?;
    cmd.wait()?;
    Ok(())
}

pub fn run(repo: &str) -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    git_clone_to(repo, tmp.path(), "test")?;

    let workdir = tmp.path().join("test");
    let moon_version = moon_version()?;
    let moonc_version = moonc_version()?;
    let rev = get_git_short_hash(&workdir)?;

    let mut logs = vec![];

    logs.extend(stat_moon(
        &workdir,
        repo,
        &rev,
        &moon_version,
        &moonc_version,
        MoonCommand::Check,
    )?);
    logs.extend(stat_moon(
        &workdir,
        repo,
        &rev,
        &moon_version,
        &moonc_version,
        MoonCommand::Build,
    )?);
    logs.extend(stat_moon(
        &workdir,
        repo,
        &rev,
        &moon_version,
        &moonc_version,
        MoonCommand::Test,
    )?);
    if repo == "https://github.com/moonbitlang/core" {
        logs.extend(stat_moon(
            &workdir,
            repo,
            &rev,
            &moon_version,
            &moonc_version,
            MoonCommand::Bundle,
        )?);
    }

    let fp = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("webapp/public/data.jsonl")?;
    let mut writer = std::io::BufWriter::new(fp);
    for log in logs {
        writeln!(writer, "{}", serde_json::to_string(&log)?)?;
    }
    writer.flush()?;
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Stat = argh::from_env();

    let mut repo_list = vec![];
    if let Some(r) = args.repo_url {
        repo_list.push(r);
    }

    if let Some(file) = &args.file {
        let content = std::fs::read_to_string(file)?;
        for line in content.lines() {
            let repo = line.trim();
            repo_list.push(repo.into());
        }
    }

    let mut results = vec![];
    for r in repo_list {
        results.push(run(&r));
    }

    for result in results {
        match result {
            Ok(()) => (),
            Err(e) => eprintln!("Error processing repository: {}", e),
        }
    }
    Ok(())
}
