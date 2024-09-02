use std::{
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use argh::FromArgs;
use chrono::{FixedOffset, Local};

use moon_dashboard::{
    git,
    util::{self, MoonCommand},
};
use moon_dashboard::{util::MooncakeSource, Statistics};

#[derive(FromArgs)]
#[argh(description = "...")]
pub struct Stat {
    #[argh(option, description = "specify a repo")]
    repo_url: Option<String>,

    #[argh(option, short = 'f', description = "read repos from file")]
    file: Option<PathBuf>,
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
            source: MooncakeSource::Git {
                url: repo.to_string(),
                rev: rev.to_string(),
            },
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

pub fn run(repo: &str) -> anyhow::Result<()> {
    let tmp = tempfile::tempdir()?;
    git::git_clone_to(repo, tmp.path(), "test")?;

    let workdir = tmp.path().join("test");
    let moon_version = util::moon_version()?;
    let moonc_version = util::moonc_version()?;
    let rev = git::get_git_short_hash(&workdir)?;

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
