use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::{Command, ExitStatus};
use std::path::Path;
use anyhow::Result;

/// Struct for parsing the .deps.toml file 
#[derive(Deserialize)]
struct Dependency {
    dependencies: Option<Vec<String>>,
}

/// Type for dependency entries
type Dependencies = HashMap<String, Dependency>;

/// Checks if inside .git repository
fn check_git_repository() -> Result<ExitStatus, String> {
    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--is-inside-work-tree")
        .output()
        .expect("Failed to execute git command");

    if output.status.success() {
        Ok(output.status)
    } else {
        Err("Not a git repository (or any of the parent directories): .git".to_string())
    }
}

/// Finds the latest
fn get_latest_commit(files: Vec<&str>) -> Option<String> {
    let output = Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--pretty=format:%H")
        .args(&["--"])
        .args(files)
        .output()
        .expect("Failed to execute git command");

    if output.status.success() {
        let commit_hash = String::from_utf8_lossy(&output.stdout);
        Some(commit_hash.to_string())
    } else {
        None
    }
}

/// Parses a file called .deps.toml in the local directory
/// If no file is found, the complete local directory (and all subdirectories) are used for the git log command
/// If the file under question does not have a .deps.toml entry, the complete local directory
/// (and all subdirectories) are used for the git log command.
/// If the file is not yet commited, the complete local directory (and all subdirectories) are used for the
/// git log command
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    check_git_repository()?;

    let filename = &args[1];
    let dependencies_path = Path::new(".deps.toml");

    // If there is no .deps.toml file, just use the local folder
    let dependencies: Dependencies = if dependencies_path.exists() {
        let toml_content = fs::read_to_string(dependencies_path)
            .expect("Failed to read .deps.toml");
        toml::from_str(&toml_content)
            .expect("Failed to parse .deps.toml")
    } else {
        HashMap::new()
    };

    let all_files: Vec<&str> = {
        let mut files = vec![filename.as_str()]; // Always include the filename itself
        if let Some(dep) = dependencies.get(filename) {
            if let Some(deps) = &dep.dependencies {
                files.extend(deps.iter().map(|s| s.as_str()));
            }
        } else if dependencies.is_empty() || !dependencies.contains_key(filename) {
            files = vec!["."];
        }
        files
    };

    let latest_commit = get_latest_commit(all_files).filter(|s| !s.is_empty());

    if let Some(commit_hash) = latest_commit {
        //println!("Latest commit affecting {}: {}", filename, commit_hash);
        println!("{}", commit_hash);
    } else {
        eprintln!("No commits found.");
        std::process::exit(1);
    }
    Ok(())
}
