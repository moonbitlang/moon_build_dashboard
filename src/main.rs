use std::{
    io::Write,
    path::Path,
    time::{Duration, Instant},
};

use chrono::{FixedOffset, Local};

use clap::Parser;
use colored::Colorize;
use moon_dashboard::{
    cli,
    dashboard::{
        Backend, BackendState, BuildState, ExecuteResult, MoonBuildDashboard, MoonCommand,
        MooncakeSource, Status, ToolChainLabel, ToolChainVersion, CBT,
    },
    mooncakesio,
    util::{get_moon_version, get_moonc_version, install_bleeding_release, install_stable_release},
};
use moon_dashboard::{git, util::moon_update};

fn run_moon(workdir: &Path, source: &MooncakeSource, args: &[&str]) -> anyhow::Result<Duration> {
    let start = Instant::now();
    eprintln!(
        "{}",
        format!("RUN moon {} for {:?}", args.join(" "), source)
            .blue()
            .bold()
    );
    let mut cmd = std::process::Command::new("moon")
        .current_dir(workdir)
        .args(args)
        .spawn()?;
    let exit = cmd.wait()?;
    if !exit.success() {
        return Err(anyhow::anyhow!("failed to execute"));
    }
    let elapsed = start.elapsed();
    eprintln!(
        "{}",
        format!(
            "moon {}, elapsed: {}ms",
            args.join(" ").blue().bold(),
            elapsed.as_millis()
        )
        .green()
        .bold()
    );
    Ok(elapsed)
}

fn get_mooncake_sources(cmd: &cli::StatSubcommand) -> anyhow::Result<Vec<MooncakeSource>> {
    let mut repo_list = vec![];
    if let Some(r) = &cmd.repo_url {
        repo_list.push(MooncakeSource::Git {
            url: r.clone(),
            rev: vec![],
            index: 0,
        });
    }

    if let Some(file) = &cmd.file {
        let content = std::fs::read_to_string(file)?;
        for line in content.lines() {
            let s = line.trim();
            if s.starts_with("https://") {
                // https://github.com/moonbitlang/core
                // https://github.com/moonbitlang/core hash1 hash2 hash3
                let parts: Vec<&str> = s.split(' ').collect();
                if parts.len() == 1 {
                    repo_list.push(MooncakeSource::Git {
                        url: parts[0].to_string(),
                        rev: vec![],
                        index: repo_list.len(),
                    });
                } else {
                    repo_list.push(MooncakeSource::Git {
                        url: parts[0].to_string(),
                        rev: parts[1..].iter().copied().map(|s| s.to_string()).collect(),
                        index: repo_list.len(),
                    });
                }
            } else {
                // moonbitlang/core
                // moonbitlang/core 0.1.0 0.2.0
                let parts: Vec<&str> = s.split(' ').collect();
                let name = parts[0].to_string();
                let version: Vec<String> =
                    parts[1..].iter().copied().map(|s| s.to_string()).collect();
                repo_list.push(MooncakeSource::MooncakesIO {
                    name,
                    version,
                    index: repo_list.len(),
                });
            }
        }
    }
    Ok(repo_list)
}

fn stat_mooncake(
    workdir: &Path,
    source: &MooncakeSource,
    cmd: MoonCommand,
) -> anyhow::Result<ExecuteResult> {
    let _ = run_moon(workdir, source, &["clean"]);

    let r = run_moon(workdir, source, &cmd.args());
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
    let mut cbts = vec![];
    match source {
        MooncakeSource::Git { url, rev, index: _ } => {
            git::git_clone_to(url, tmp.path(), "test")?;
            let workdir = tmp.path().join("test");
            for h in rev {
                git::git_checkout(&workdir, h)?;
                let check_wasm =
                    stat_mooncake(&workdir, source, MoonCommand::Check(Backend::Wasm))?;
                let check_wasm_gc =
                    stat_mooncake(&workdir, source, MoonCommand::Check(Backend::WasmGC))?;
                let check_js = stat_mooncake(&workdir, source, MoonCommand::Check(Backend::Js))?;

                let build_wasm =
                    stat_mooncake(&workdir, source, MoonCommand::Build(Backend::Wasm))?;
                let build_wasm_gc =
                    stat_mooncake(&workdir, source, MoonCommand::Build(Backend::WasmGC))?;
                let build_js = stat_mooncake(&workdir, source, MoonCommand::Build(Backend::Js))?;

                let test_wasm = stat_mooncake(&workdir, source, MoonCommand::Test(Backend::Wasm))?;
                let test_wasm_gc =
                    stat_mooncake(&workdir, source, MoonCommand::Test(Backend::WasmGC))?;
                let test_js = stat_mooncake(&workdir, source, MoonCommand::Test(Backend::Js))?;

                cbts.push(CBT {
                    check: BackendState {
                        wasm: check_wasm,
                        wasm_gc: check_wasm_gc,
                        js: check_js,
                    },
                    build: BackendState {
                        wasm: build_wasm,
                        wasm_gc: build_wasm_gc,
                        js: build_js,
                    },
                    test: BackendState {
                        wasm: test_wasm,
                        wasm_gc: test_wasm_gc,
                        js: test_js,
                    },
                });
            }
        }
        MooncakeSource::MooncakesIO {
            name,
            version,
            index: _,
        } => {
            for v in version {
                mooncakesio::download_to(name, &v, tmp.path())?;
                let workdir = tmp.path().join(v);
                let check_wasm =
                    stat_mooncake(&workdir, source, MoonCommand::Check(Backend::Wasm))?;
                let check_wasm_gc =
                    stat_mooncake(&workdir, source, MoonCommand::Check(Backend::WasmGC))?;
                let check_js = stat_mooncake(&workdir, source, MoonCommand::Check(Backend::Js))?;

                let build_wasm =
                    stat_mooncake(&workdir, source, MoonCommand::Build(Backend::Wasm))?;
                let build_wasm_gc =
                    stat_mooncake(&workdir, source, MoonCommand::Build(Backend::WasmGC))?;
                let build_js = stat_mooncake(&workdir, source, MoonCommand::Build(Backend::Js))?;

                let test_wasm = stat_mooncake(&workdir, source, MoonCommand::Test(Backend::Wasm))?;
                let test_wasm_gc =
                    stat_mooncake(&workdir, source, MoonCommand::Test(Backend::WasmGC))?;
                let test_js = stat_mooncake(&workdir, source, MoonCommand::Test(Backend::Js))?;

                cbts.push(CBT {
                    check: BackendState {
                        wasm: check_wasm,
                        wasm_gc: check_wasm_gc,
                        js: check_js,
                    },
                    build: BackendState {
                        wasm: build_wasm,
                        wasm_gc: build_wasm_gc,
                        js: build_js,
                    },
                    test: BackendState {
                        wasm: test_wasm,
                        wasm_gc: test_wasm_gc,
                        js: test_js,
                    },
                });
            }
        }
    }

    Ok(BuildState {
        source: source.get_index(),
        cbts,
    })
}

fn stat(cmd: cli::StatSubcommand) -> anyhow::Result<MoonBuildDashboard> {
    let run_id = std::env::var("GITHUB_ACTION_RUN_ID").unwrap_or("0".into());
    let run_number = std::env::var("GITHUB_ACTION_RUN_NUMBER").unwrap_or("0".into());

    install_stable_release()?;
    moon_update()?;
    let moon_version = get_moon_version()?;
    let moonc_version = get_moonc_version()?;
    let stable_toolchain_version = ToolChainVersion {
        label: ToolChainLabel::Stable,
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
    moon_update()?;
    let moon_version = get_moon_version()?;
    let moonc_version = get_moonc_version()?;
    let bleeding_toolchain_version = ToolChainVersion {
        label: ToolChainLabel::Bleeding,
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
            let fp = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("webapp/public/data.jsonl")?;
            let mut writer = std::io::BufWriter::new(fp);
            writeln!(writer, "{}", serde_json::to_string(&dashboard)?)?;
            writer.flush()?;
            Ok(())
        }
        Err(e) => Err(e),
    }
}

fn main() -> anyhow::Result<()> {
    main0()
}
