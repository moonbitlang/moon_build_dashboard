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
