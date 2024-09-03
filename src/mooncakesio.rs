use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://moonbitlang-mooncakes.s3.us-west-2.amazonaws.com/user";

pub fn download_to(name: &str, version: &str, dst: &Path) -> anyhow::Result<()> {
    let version_enc = form_urlencoded::Serializer::new(String::new())
        .append_key_only(&version)
        .finish();
    let url = format!("{}/{}/{}.zip", BASE_URL, name, version_enc);
    let output_zip = format!("{}.zip", dst.join(version).display());
    let output = std::process::Command::new("curl")
        .arg("-o")
        .arg(&output_zip)
        .arg(&url)
        .output()?;
    if !output.status.success() {
        anyhow::bail!("failed to download {}", url)
    }

    let output = std::process::Command::new("unzip")
        .arg(&output_zip)
        .arg("-d")
        .arg(dst.join(version))
        .output()?;
    if !output.status.success() {
        anyhow::bail!("failed to unzip {}", output_zip)
    }

    Ok(())
}

pub fn home() -> PathBuf {
    if let Ok(moon_home) = std::env::var("MOON_HOME") {
        return PathBuf::from(moon_home);
    }

    let h = home::home_dir();
    if h.is_none() {
        eprintln!("Failed to get home directory");
        std::process::exit(1);
    }
    let hm = h.unwrap().join(".moon");
    if !hm.exists() {
        std::fs::create_dir_all(&hm).unwrap();
    }
    hm
}

pub fn index() -> PathBuf {
    home().join("registry").join("index")
}

pub fn index_of_pkg(base: &Path, user: &str, pkg: &str) -> PathBuf {
    base.join("user")
        .join(user)
        .join(pkg)
        .with_extension("index")
}

#[derive(Debug)]
pub struct MooncakesDB {
    db: BTreeMap<String, Vec<String>>,
}

impl MooncakesDB {
    pub fn new() -> Self {
        Self {
            db: BTreeMap::new(),
        }
    }

    pub fn get_latest_version(&self, name: &str) -> Option<&String> {
        self.db.get(name).map(|versions| versions.last().unwrap())
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct MooncakeInfo {
    version: String,
    keywords: Option<Vec<String>>,
}

#[test]
fn gen_latest_list() {
    let db = get_all_mooncakes().unwrap();
    for (name, versions) in db.db {
        let latest_version = versions.last().unwrap();
        println!("{} {}", name, latest_version);
    }
}

#[test]
fn gen_latest_list_with_version() {
    let db = get_all_mooncakes().unwrap();
    for (name, versions) in db.db {
        let latest_version = versions.last().unwrap();
        println!("{} latest {}", name, latest_version);
    }
}

pub fn get_all_mooncakes() -> anyhow::Result<MooncakesDB> {
    let mut db: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let dir = index().join("user");
    let walker = walkdir::WalkDir::new(dir).into_iter();
    for entry in walker.filter_map(|e| e.ok()).filter(|e| {
        e.path().is_file() && e.path().extension().and_then(|ext| ext.to_str()) == Some("index")
    }) {
        let username = entry
            .path()
            .parent()
            .unwrap()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();
        let pkgname = entry.path().file_stem().unwrap().to_str().unwrap();
        let name = format!("{}/{}", username, pkgname);
        let index_file_content = std::fs::read_to_string(entry.path())?;
        for line in index_file_content.lines() {
            let index: MooncakeInfo = serde_json::from_str(line)?;
            if let Some(keywords) = &index.keywords {
                if keywords.contains(&"mooncakes-test".to_string()) {
                    continue;
                }
            }
            db.entry(name.to_string()).or_default().push(index.version);
        }
    }
    Ok(MooncakesDB { db })
}
