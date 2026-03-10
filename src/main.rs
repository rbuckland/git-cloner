use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use structopt::StructOpt;
use url::ParseError;
use url::Url;

use std::io::{BufRead, BufReader};

fn get_default_workspace() -> String {
    std::env::var("CLONER_WORKSPACE").unwrap_or_else(|_| "~/projects".to_string())
}

fn infer_host_from_cwd(workspace: &str) -> Option<String> {
    let cwd = std::env::current_dir().ok()?;

    let workspace_expanded = shellexpand::tilde(workspace).to_string();
    let workspace_path = Path::new(&workspace_expanded);

    // Check if current directory is under the workspace
    if let Ok(relative) = cwd.strip_prefix(workspace_path) {
        let components: Vec<_> = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        // Return host if we have at least one component
        if !components.is_empty() {
            return Some(components[0].clone());
        }
    }

    None
}

fn infer_host_org_from_cwd(workspace: &str) -> Option<(String, String)> {
    let cwd = std::env::current_dir().ok()?;

    let workspace_expanded = shellexpand::tilde(workspace).to_string();
    let workspace_path = Path::new(&workspace_expanded);

    // Check if current directory is under the workspace
    if let Ok(relative) = cwd.strip_prefix(workspace_path) {
        let components: Vec<_> = relative
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect();

        // Need at least host/org in the path
        if components.len() >= 2 {
            let host = components[0].clone();
            let org = components[1].clone();
            return Some((host, org));
        }
    }

    None
}

fn parse_command_line_repo(src: &str) -> Result<Url, ParseError> {
    // If it's already a valid URL, return it
    if let Ok(url) = Url::parse(src) {
        if url.scheme() == "http" || url.scheme() == "https" || url.scheme() == "ssh" {
            return Ok(url);
        }
    }

    // Try to infer from current directory
    let workspace = get_default_workspace();
    if let Some((host, org)) = infer_host_org_from_cwd(&workspace) {
        // Handle simple repo name like "stm.aux"
        if !src.contains('/') {
            let url_string = format!("https://{}/{}/{}", host, org, src);
            return Url::parse(&url_string);
        }
        // Handle org/repo format like "sqc-internal/stm.aux"
        else if src.split('/').count() == 2 {
            let url_string = format!("https://{}/{}", host, src);
            return Url::parse(&url_string);
        }
    } else if src.split('/').count() == 2 {
        // If we can't infer host/org, but we have org/repo format, try to infer just the host
        if let Some(host) = infer_host_from_cwd(&workspace) {
            let url_string = format!("https://{}/{}", host, src);
            return Url::parse(&url_string);
        }
    }

    // Try as-is
    Url::parse(src)
}

/// Command line options
#[derive(StructOpt, Debug)]
#[structopt(name = "git-cloner", about = "Clone git repositories into organized workspace")]
struct Args {
    /// Enable completion mode - list repositories in the current org
    #[structopt(long)]
    complete: bool,

    /// Clone a repository (default behavior)
    #[structopt(long)]
    clone: bool,

    /// Dry run - show what would be done without executing
    #[structopt(short, long)]
    dry_run: bool,

    /// Workspace root directory
    #[structopt(long, default_value = "~/projects", env = "CLONER_WORKSPACE")]
    workspace: String,

    /// Repository URL, org/repo, or repo name
    #[structopt(parse(try_from_str = parse_command_line_repo))]
    url: Option<Url>,
}

/// assume the last option is the repo name
fn repo_from_url(url: &Url) -> &str {
    url.path_segments()
        .unwrap()
        .next_back()
        .map(|segment| {
            if let Some(stripped) = segment.strip_suffix(".git") {
                stripped
            } else {
                segment
            }
        })
        .unwrap_or_default()
}

// assumes the first element is the org (only supports one level right now)
fn org_from_url(url: &Url) -> PathBuf {
    let path_segments: Vec<&str> = url.path_segments().unwrap().collect();
    let mut org_path = PathBuf::new();

    // bitbucket style
    // Check if the root path starts with "scm" and has 3 or more segments
    if path_segments.len() >= 3 && path_segments[0] == "scm" {
        for ps in path_segments.iter().take(path_segments.len() - 1).skip(1) {
            org_path.push(ps);
        }
    }
    // subprojects on gitlab, style
    // normal projects on github.com
    else {
        for ps in path_segments.iter().take(path_segments.len() - 1) {
            org_path.push(ps);
        }
    }

    org_path
}

/// the host portion of the URL
fn hostname_from_url(url: &Url) -> &str {
    url.host_str().unwrap_or_default()
}

/// construct the path to clone to
fn get_site_root_folder(workspace: &str, url: &Url) -> PathBuf {
    let workspace = shellexpand::tilde(workspace).to_string();
    let host = hostname_from_url(url);
    let org = org_from_url(url);
    let folder = Path::new(&workspace)
        .join(host)
        .join(org)
        .to_string_lossy()
        .into_owned();
    PathBuf::from(folder)
}

// fn list_github_org_repos_url(org: &str) -> Result<Vec<String>> {
//     println!("Listing repositories for organization: {}", org);
//     let url = format!("https://api.github.com/orgs/{}/repos?per_page=20", org);
//     let client = reqwest::blocking::Client::new();
//     let res = client
//         .get(&url)
//         .header("User-Agent", "git-cloner")
//         .send()?
//         .json::<serde_json::Value>()?;

//     // debug the response
//     println!("Response: {:?}", res);

//     if !res.is_array() {
//         return Err(anyhow::anyhow!("Unexpected response format from GitHub API"));
//     }

//     let mut repos = Vec::new();
//     if let Some(arr) = res.as_array() {
//         for repo in arr {
//             if let Some(name) = repo.get("name").and_then(|n| n.as_str()) {
//                 repos.push(name.to_string());
//             }
//         }
//     }
//     Ok(repos)
// }

fn list_github_org_repos_gh_cli(org: &str) -> Result<Vec<String>> {
    let output = Command::new("gh")
        .arg("repo")
        .arg("list")
        .arg("--json")
        .arg("name")
        // .arg("--jq")
        // .arg("[.[] | .name ]")
        .arg(org)
        .arg("--limit")
        .arg("50")
        .output()?;
    if !output.status.success() {
        return Err(anyhow::anyhow!(
            "Failed to list repositories for organization: {}",
            org
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let res: serde_json::Value = serde_json::from_str(&stdout)?;

    if !res.is_array() {
        return Err(anyhow::anyhow!("Unexpected response format from gh CLI"));
    }
    // Extract repository names from the JSON response

    let mut repos = Vec::new();
    if let Some(arr) = res.as_array() {
        for repo in arr {
            if let Some(name) = repo.get("name").and_then(|n| n.as_str()) {
                repos.push(name.to_string());
            }
        }
    }
    Ok(repos)
}

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();

    // Handle completion mode
    if args.complete {
        if let Some((host, org)) = infer_host_org_from_cwd(&args.workspace) {
            if host == "github.com" {
                let repos = list_github_org_repos_gh_cli(&org)?;
                for repo in repos {
                    println!("{}", repo);
                }
            } else {
                eprintln!("Completion only supported for github.com orgs");
            }
        } else {
            eprintln!("Could not infer host/org from CWD");
        }
        return Ok(());
    }

    // Handle clone mode (default)
    let url = args.url.unwrap_or_else(|| {
        eprintln!("error: No repository URL provided");
        eprintln!();
        eprintln!("USAGE:");
        eprintln!("    git-cloner <repo-url>");
        eprintln!("    git-cloner --complete");
        eprintln!();
        eprintln!("For more information try --help");
        process::exit(1);
    });

    let folder = get_site_root_folder(&args.workspace, &url);
    let full_path = &folder.clone().join(repo_from_url(&url));

    println!(
        "» Cloning {} → {}",
        &url,
        &full_path.clone().into_os_string().into_string().unwrap()
    );

    if args.dry_run {
        println!("» mkdir -p {:?}", &folder);
        println!("» git clone {}", &url);
    } else {
        fs::create_dir_all(&folder).expect("!! Failed to create directories");
        let mut cmd = Command::new("git")
            .arg("clone")
            .arg("--progress")
            .arg(url.as_str())
            .current_dir(&folder)
            .stdout(Stdio::piped())
            .spawn()?;

        let stdout = cmd.stdout.take().expect("Failed to open stdout");
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            println!("{}", line?);
        }

        let status = cmd.wait()?;
        if status.success() {
            println!(
                "» Cloned to {}",
                &full_path.clone().into_os_string().into_string().unwrap()
            );
        }
        process::exit(status.code().unwrap_or(1));
    }

    Ok(())
}
