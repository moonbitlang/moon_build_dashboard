use std::io::Write;

pub fn get_moon_version() -> anyhow::Result<String> {
    let output = std::process::Command::new("moon")
        .args(["version"])
        .output()?;
    let version = String::from_utf8(output.stdout)?;
    Ok(version.trim().to_string())
}

pub fn get_moonc_version() -> anyhow::Result<String> {
    let output = std::process::Command::new("moonc").args(["-v"]).output()?;
    let version = String::from_utf8(output.stdout)?;
    Ok(version.trim().to_string())
}

fn install_release(args: &[&str]) -> anyhow::Result<()> {
    let output = std::process::Command::new("curl")
        .args(["-fsSL", "https://cli.moonbitlang.com/install/unix.sh"])
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to download the installation script"
        ));
    }

    let mut cmd = std::process::Command::new("bash")
        .args(args)
        .stdin(std::process::Stdio::piped())
        .spawn()?;

    if let Some(stdin) = cmd.stdin.as_mut() {
        stdin.write_all(&output.stdout)?;
    }

    let status = cmd.wait()?;
    if !status.success() {
        return Err(anyhow::anyhow!("Failed to execute the installation script"));
    }

    Ok(())
}

pub fn install_stable_release() -> anyhow::Result<()> {
    install_release(&["-s"])
}

pub fn install_bleeding_release() -> anyhow::Result<()> {
    install_release(&["-s", "bleeding"])
}

pub fn moon_update() -> anyhow::Result<()> {
    let output = std::process::Command::new("moon")
        .args(["update"])
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!("Failed to update moon"));
    }
    Ok(())
}
