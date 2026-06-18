use std::fs;
use std::path::PathBuf;
use crate::error::{HarError, Result};

const SKILL: &str = include_str!("../../skills/analyze-har/CLAUDE.md");

pub fn run(global: bool) -> Result<()> {
    let skills_dir = if global {
        home_dir()?.join(".claude").join("skills")
    } else {
        PathBuf::from(".claude").join("skills")
    };
    let dest_dir = skills_dir.join("analyze-har");
    fs::create_dir_all(&dest_dir)?;
    let dest = dest_dir.join("CLAUDE.md");
    fs::write(&dest, SKILL)?;
    println!("installed: {}", dest.display());
    Ok(())
}

fn home_dir() -> Result<PathBuf> {
    #[cfg(windows)]
    let var = "USERPROFILE";
    #[cfg(not(windows))]
    let var = "HOME";
    std::env::var_os(var)
        .map(PathBuf::from)
        .ok_or_else(|| HarError::Usage("cannot determine home directory".to_string()))
}
