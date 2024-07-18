use log::{self, debug, error, info, warn};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
};

const DEPENDENCIES_PATH: &str = ".deps.toml";

const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if the working tree is clean, returns "" or " DIRTY"
fn check_clean_working_tree(files: &Vec<String>, cwd: &Path) -> String {
    let output = Command::new("git")
        .current_dir(cwd)
        .arg("status")
        .arg("--porcelain=v2") // stable scripting interface
        .args(files)
        .output()
        .unwrap();

    // if there is no output, working tree is clean
    if output.stdout == vec![] {
        "".to_string()
    } else {
        " DIRTY".to_string() // otherwise DIRTY will be appended
    }
}

/// Checks if inside .git repository.
/// Theoretically redundant, only for nicer error messages.
fn check_git_repository(cwd: &Path) -> Result<ExitStatus, String> {
    let output = Command::new("git")
        .current_dir(cwd)
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

/// Finds the latest commit affecting the files, or the date of this latest commit
fn get_latest_commit(files: &Vec<String>, get_date: bool, cwd: &Path) -> Option<String> {
    let format = if get_date {
        "--pretty=format:%cs"
    } else {
        "--pretty=format:%H"
    }; // cs is commiter date, short format: https://git-scm.com/docs/pretty-formats
    let output = Command::new("git")
        .current_dir(cwd)
        .arg("log")
        .arg("-1")
        .arg(format)
        .arg("--")
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

/// Parses a file called .deps.toml in the local directory.
/// If no file is found, the complete local directory (and all subdirectories) are used for the git log command
/// If the file under question does not have a .deps.toml entry, the complete local directory
/// (and all subdirectories) are used for the git log command.
/// If the file is not yet commited, the complete local directory (and all subdirectories) are used for the
/// git log command.
fn main() {
    simple_logger::init().unwrap();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 || args.len() > 3 {
        eprintln!("Usage: {} <filename> [--date]", args[0]);
        std::process::exit(1);
    }

    if &args[1] == "-v" || &args[1] == "--version" {
        eprintln!("Version: {}", VERSION);
        std::process::exit(0);
    }

    let filepath = PathBuf::from(&args[1])
        .canonicalize()
        .unwrap_or_else(|e| panic!("Invalid file: {}. Error: {}", &args[1], e));

    let base_directory = if filepath.is_dir() {
        &filepath
    } else {
        filepath
            .parent()
            .expect("Cannot obtain directory for filename")
    };

    let base_directory_string = base_directory
        .to_str()
        .expect("Cannot convert base directory to string");

    debug!("Using cwd: {:#?}", base_directory);

    check_git_repository(base_directory).expect("Checking git repository failed");

    // Check if the file exists at all
    filepath
        .try_exists()
        .unwrap_or_else(|_| panic!("{} does not exist", filepath.display()));

    let filename = filepath
        .file_name()
        .expect("Could not obtain filename from filepath.")
        .to_str()
        .expect("filename not convertible to string");

    info!("Monitor changes for file: {:#?}", filepath);

    let get_date = args.get(2).map_or(false, |arg| arg == "--date");

    let dependencies_path = base_directory.join(DEPENDENCIES_PATH);

    let dependencies = if dependencies_path.exists() {
        let toml_file_string =
            fs::read_to_string(&dependencies_path).expect("Failed to read .deps.toml");
        let toml_file_table: toml::map::Map<String, toml::Value> = toml_file_string
            .parse::<toml::Table>()
            .expect("Failed to parse .deps.toml");

        let dependencies: Option<Vec<String>> = toml_file_table
            .get(filename)
            .and_then(|key| key.get("dependencies"))
            .and_then(|deps| deps.as_array())
            .map(|deps| {
                deps.iter()
                    .map(|dep| {
                        dep.as_str()
                            .expect("dependency was not a string")
                            .to_string()
                    })
                    .collect()
            });
        dependencies
    } else {
        None
    };

    debug!(
        "Searching: {:#?}. Found dependencies: {:#?}",
        dependencies_path, dependencies,
    );

    // Collect a Vec of all files that shall be monitored.
    // First, determine whether any dependencies for the file are specified.
    // This is nested in one extra struct so we can extend this later on without breaking the existing toml files.
    let all_files = match dependencies {
        Some(deps) => {
            let mut files = vec![filename.to_string()]; // Always include the filename itself
            files.extend(deps.into_iter().map(|dep| dep.to_string()));
            files
        }
        None => {
            // If the given filename hasn't been specified in the toml file, we just we watch the file's base_directory.
            warn!(
                "No dependencies entry found for file {:#?}. Monitoring basedirectory.",
                filename
            );
            vec![base_directory_string.to_string()]
        }
    };

    debug!("Files monitored for changes: {:#?}", all_files);

    let latest_commit =
        get_latest_commit(&all_files, get_date, base_directory).filter(|s| !s.is_empty());

    if let Some(commit_hash) = latest_commit {
        debug!("Latest commit affecting {:#?}: {}", all_files, commit_hash);
        let mut owned_hash: String = commit_hash.to_owned();
        if !get_date {
            let owned_flag: String =
                check_clean_working_tree(&all_files, base_directory).to_owned();
            owned_hash.push_str(&owned_flag);
        }
        println!("{owned_hash}");
    } else {
        error!("No commits found.");
        std::process::exit(1);
    }
}
