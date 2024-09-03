use std::path::Path;

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
        .args(["clone", repo, dst])
        .spawn()?;
    cmd.wait()?;
    Ok(())
}

pub fn git_checkout(workdir: &Path, rev: &str) -> anyhow::Result<()> {
    let mut cmd = std::process::Command::new("git")
        .current_dir(workdir)
        .args(["checkout", rev])
        .spawn()?;
    cmd.wait()?;
    Ok(())
}
