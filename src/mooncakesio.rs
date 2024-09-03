use std::path::Path;

const BASE_URL: &str = "https://moonbitlang-mooncakes.s3.us-west-2.amazonaws.com/user";

pub fn download_to(name: &str, version: &str, dst: &Path) -> anyhow::Result<()> {
    let url = format!("{}/{}/{}.zip", BASE_URL, name, version);
    let output = std::process::Command::new("curl")
        .arg("-o")
        .arg(dst.join(version).with_extension("zip"))
        .arg(&url)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("failed to download {}", url)
    }

    let output = std::process::Command::new("unzip")
        .arg(dst.join(version).with_extension("zip"))
        .arg("-d")
        .arg(dst.join(version))
        .output()?;
    if !output.status.success() {
        anyhow::bail!(
            "failed to unzip {}",
            dst.join(version).with_extension("zip").display()
        )
    }

    Ok(())
}
