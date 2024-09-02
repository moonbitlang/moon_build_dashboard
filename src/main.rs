use std::{
    path::Path,
    time::{Duration, Instant},
};

use chrono::{FixedOffset, Local};

use clap::Parser;
use moon_dashboard::git;
use moon_dashboard::{
    cli,
    dashboard::{
        BuildState, ExecuteResult, MoonBuildDashboard, MoonCommand, MooncakeSource, Status,
        ToolChainVersion,
    },
    util::{get_moon_version, get_moonc_version, install_bleeding_release, install_stable_release},
};

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

fn get_mooncake_sources(cmd: &cli::StatSubcommand) -> anyhow::Result<Vec<MooncakeSource>> {
    let mut repo_list = vec![];
    if let Some(r) = &cmd.repo_url {
        repo_list.push(MooncakeSource::Git {
            url: r.clone(),
            rev: None,
        });
    }

    if let Some(file) = &cmd.file {
        let content = std::fs::read_to_string(file)?;
        for line in content.lines() {
            let repo = line.trim();
            if !repo.is_empty() {
                repo_list.push(MooncakeSource::Git {
                    url: repo.to_string(),
                    rev: None,
                });
            }
        }
    }
    Ok(repo_list)
}

fn stat_mooncake(workdir: &Path, cmd: MoonCommand) -> anyhow::Result<ExecuteResult> {
    let _ = run_moon(workdir, &["clean"]);

    let r = run_moon(workdir, &cmd.args());
    let status = if r.is_err() {
        Status::Failure
    } else {
        Status::Success
    };
    let d = r.ok();
    let start_time = Local::now()
        .with_timezone(&FixedOffset::east_opt(8 * 3600).unwrap())
        .format("%Y-%m-%d %H:%M:%S.%3f")
        .to_string();
    let elapsed = d.map(|d| d.as_millis() as u64).unwrap_or(0);
    let execute_result = ExecuteResult {
        status,
        start_time,
        elapsed,
    };
    Ok(execute_result)
}

pub fn build(source: &MooncakeSource) -> anyhow::Result<BuildState> {
    let tmp = tempfile::tempdir()?;
    match source {
        MooncakeSource::Git { url, rev: _ } => {
            git::git_clone_to(url, tmp.path(), "test")?;
        }
        MooncakeSource::MooncakesIO { .. } => {
            todo!()
        }
    }
    let workdir = tmp.path().join("test");
    let check = stat_mooncake(&workdir, MoonCommand::Check)?;
    let build = stat_mooncake(&workdir, MoonCommand::Build)?;
    let test = stat_mooncake(&workdir, MoonCommand::Test)?;
    Ok(BuildState {
        source: source.clone(),
        check,
        build,
        test,
    })
}

fn stat(cmd: cli::StatSubcommand) -> anyhow::Result<MoonBuildDashboard> {
    let run_id = std::env::var("GITHUB_ACTION_RUN_ID").unwrap_or("0".into());
    let run_number = std::env::var("GITHUB_ACTION_RUN_NUMBER").unwrap_or("0".into());

    install_stable_release()?;
    let moon_version = get_moon_version()?;
    let moonc_version = get_moonc_version()?;
    let stable_toolchain_version = ToolChainVersion {
        label: "stable".into(),
        moon_version,
        moonc_version,
    };

    let mooncake_sources = get_mooncake_sources(&cmd)?;
    let mut stable_release_data = vec![];

    for source in mooncake_sources {
        let build_state = build(&source)?;
        stable_release_data.push(build_state);
    }

    install_bleeding_release()?;
    let moon_version = get_moon_version()?;
    let moonc_version = get_moonc_version()?;
    let bleeding_toolchain_version = ToolChainVersion {
        label: "bleeding".into(),
        moon_version,
        moonc_version,
    };

    let mooncake_sources = get_mooncake_sources(&cmd)?;
    let mut bleeding_release_data = vec![];

    for source in mooncake_sources.iter() {
        let build_state = build(source)?;
        bleeding_release_data.push(build_state);
    }

    let result = MoonBuildDashboard {
        run_id,
        run_number,
        sources: mooncake_sources,
        start_time: Local::now().to_rfc3339(),
        stable_toolchain_version,
        stable_release_data,
        bleeding_toolchain_version,
        bleeding_release_data,
    };
    dbg!(&result);
    Ok(result)
}

fn main0() -> anyhow::Result<()> {
    let cli = cli::MoonBuildDashBoardCli::parse();
    let res = match cli.subcommand {
        cli::MoonBuildDashBoardSubcommands::Stat(cmd) => stat(cmd),
    };
    match res {
        Ok(dashboard) => {
            println!("{}", serde_json::to_string(&dashboard)?);
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn main() -> anyhow::Result<()> {
    main0()
}
