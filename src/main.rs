use anyhow::Result;
use directories::UserDirs;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use structopt::StructOpt;
use url::ParseError;
use url::Url;

use std::io::{self, BufRead, BufReader};

fn parse_url(src: &str) -> Result<Url, ParseError> {
    Url::parse(src)
}

#[derive(StructOpt, Debug)]
struct Args {
    #[structopt(short, long)]
    dry_run: bool,

    #[structopt(long, default_value = "~/projects", env = "CLONER_WORKSPACE")]
    workspace: String,

    #[structopt(parse(try_from_str = parse_url))]
    url: Url,
}

fn repo_from_url(url: &Url) -> &str {
    url.path_segments().unwrap().last().unwrap_or_default()
}

fn org_from_url(url: &Url) -> &str {
    url.path_segments().unwrap().nth(0).unwrap_or_default()
}

fn hostname_from_url(url: &Url) -> &str {
    url.host_str().unwrap_or_default()
}

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