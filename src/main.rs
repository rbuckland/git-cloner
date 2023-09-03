use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use structopt::StructOpt;
use url::ParseError;
use url::Url;

use std::io::{BufRead, BufReader};

fn parse_url(src: &str) -> Result<Url, ParseError> {
    Url::parse(src)
}

/// Command line options
#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short, long)]
    dry_run: bool,

    #[structopt(long, default_value = "~/projects", env = "CLONER_WORKSPACE")]
    workspace: String,

    #[structopt(parse(try_from_str = parse_url))]
    url: Url,
}
/// assume the last option is the repo name
fn repo_from_url(url: &Url) -> &str {
    url.path_segments()
        .unwrap()
        .last()
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

fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    let folder = get_site_root_folder(&args.workspace, &args.url);

    let full_path = &folder.clone().join(repo_from_url(&args.url));
    println!(
        "» Cloning {} → {}",
        &args.url,
        &full_path.clone().into_os_string().into_string().unwrap()
    );

    if args.dry_run {
        println!("» mkdir -p {:?}", &folder);
        println!("» git clone {}", args.url);
    } else {
        fs::create_dir_all(&folder).expect("!! Failed to create directories");
        let mut cmd = Command::new("git")
            .arg("clone")
            .arg("--progress")
            .arg(args.url.as_str())
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
    };

    Ok(())
}
