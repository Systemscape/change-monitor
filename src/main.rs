use log::{self, debug, error, info, warn};
use serde::Deserialize;
use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
    process::{Command, ExitStatus},
};

const DEPENDENCIES_PATH: &str = ".deps.toml";

/// Struct for parsing the .deps.toml file
#[derive(Deserialize, Debug, Clone)]
struct Dependency {
    dependencies: Option<Vec<PathBuf>>,
}

/// Type for dependency entries
type Dependencies = HashMap<String, Dependency>;

/// Check if the working tree is clean, returns "" or " DIRTY"
fn check_clean_working_tree(files: &Vec<PathBuf>) -> String {
    let output = Command::new("git")
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

/// Finds the latest commit affecting the files, or the date of this latest commit
fn get_latest_commit(files: &Vec<PathBuf>, get_date: bool) -> Option<String> {
    let format = if get_date {
        "--pretty=format:%cs"
    } else {
        "--pretty=format:%H"
    }; // cs is commiter date, short format: https://git-scm.com/docs/pretty-formats
    let output = Command::new("git")
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

    if args.len() != 2 {
        eprintln!("Usage: {} <filename> [--date <date>]", args[0]);
        std::process::exit(1);
    }

    check_git_repository().expect("Checking git repository failed");

    let filepath = PathBuf::from(&args[1])
        .canonicalize()
        .expect("Unable to canonicalize <filename>");

    let base_directory = filepath
        .parent()
        .expect("Cannot obtain directory for filename")
        .to_path_buf();

    // Check if the file exists at all
    filepath
        .try_exists()
        .unwrap_or_else(|_| panic!("{} does not exist", filepath.display()));

    let filename = filepath
        .file_name()
        .expect("Could not obtain filename from filepath.");

    info!("Monitor changes for file: {:#?}", filepath);

    let get_date = args.get(2).map_or(false, |arg| arg == "--date");

    let dependencies_path = base_directory.join(DEPENDENCIES_PATH);

    // If there is no .deps.toml file, just use the local folder
    let dependencies: Option<Dependencies> = if dependencies_path.exists() {
        let toml_content =
            fs::read_to_string(&dependencies_path).expect("Failed to read .deps.toml");
        Some(toml::from_str(&toml_content).expect("Failed to parse .deps.toml"))
    } else {
        None
    };

    debug!(
        "Found dependencies: {:#?}\nSourcefile: {:#?}",
        dependencies,
        dependencies_path.display()
    );

    // Collect a Vec of all files that shall be monitored.
    // First, determine whether any dependencies for the file are specified.
    // This is nested in one extra struct so we can extend this later on without breaking the existing toml files.
    let all_files = match dependencies.and_then(|map| map.get(filename.to_str().unwrap()).cloned())
    {
        Some(deps) => {
            // Dependencies have been specified. So we get them from the nested struct.
            // When that nested Option is None, we continue with an empty vector (unwrap_or_default).
            let mut dependencies = deps.dependencies.unwrap_or_default();

            // For all files, check whether they exist and issue a warning if they don't.
            for file in &mut dependencies {
                match file.try_exists() {
                    Ok(result) if !result => {
                        warn!("{} does not exist. Ignoring...", file.display());
                        file.clear()
                    }
                    Err(_) => warn!("Cannot determine whether {} exist", file.display()),
                    _ => (),
                }
            }

            // Filter out files that don't exist or can't be canoncialized (bad! this should never happen for an existing file).
            // This should also filter out any non-existent files that we `clear()`ed earlier.
            dependencies = dependencies
                .iter_mut()
                .filter(|dep| dep.exists())
                .filter_map(|dep| dep.canonicalize().ok())
                .collect::<Vec<_>>();

            debug!("Dependencies after filtering: {:#?}", dependencies);

            let mut files = vec![filepath.clone()]; // Always include the filename itself
            files.extend(dependencies);
            files
        }
        None => {
            // If the given filename hasn't been specified in the toml file, we just we watch the file's base_directory.
            warn!(
                "No dependencies entry found for file {:#?}. Monitoring basedirectory.",
                filename
            );
            vec![base_directory]
        }
    };

    debug!("all_files to monitor: {:#?}", all_files);

    let latest_commit = get_latest_commit(&all_files, get_date).filter(|s| !s.is_empty());

    if let Some(commit_hash) = latest_commit {
        debug!("Latest commit affecting {:#?}: {}", all_files, commit_hash);
        let mut owned_hash: String = commit_hash.to_owned();
        if !get_date {
            let owned_flag: String = check_clean_working_tree(&all_files).to_owned();
            owned_hash.push_str(&owned_flag);
        }
        println!("{owned_hash}");
    } else {
        error!("No commits found.");
        std::process::exit(1);
    }
}
