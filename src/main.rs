use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Command;

#[derive(Deserialize)]
struct Dependency {
    dependencies: Option<Vec<String>>,
}

type Dependencies = HashMap<String, Dependency>;

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

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <filename>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let toml_content = fs::read_to_string(".deps.toml")
        .expect("Failed to read .deps.toml");
    let dependencies: Dependencies = toml::from_str(&toml_content)
        .expect("Failed to parse .dseps.toml");

    let all_files: Vec<&str> = if let Some(dependency) = dependencies.get(filename) {
        if let Some(deps) = &dependency.dependencies {
            let mut files = vec![filename.as_str()];
            files.extend(deps.iter().map(|s| s.as_str()));
            files
        } else {
            vec![filename.as_str()]
        }
    } else {
        vec![filename.as_str()]
    };

    let latest_commit = get_latest_commit(all_files)
        .or_else(|| get_latest_commit(vec!["."]));

    if let Some(commit_hash) = latest_commit {
        println!("Latest commit affecting {}: {}", filename, commit_hash);
    } else {
        eprintln!("No commits found.");
    }
}
