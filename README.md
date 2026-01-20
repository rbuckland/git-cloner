# git-cloner

## Quick Install

```bash
curl -fsSL https://rbuckland.github.io/git-cloner/install.sh | bash
```

This will:
- Install `git-cloner` to `~/.local/bin`
- Copy tab completion giles to `~/.local/share/git-cloner`
- Setup the `git cloner` alias in `git config --global`
- Configure tab completion for bash/zsh

## TL;DR

```
❯ git cloner https://github.com/inosion/madato
» Cloning https://github.com/inosion/madato → /home/rbuckland/projects/github.com/inosion/madato
Cloning into 'madato'...
remote: Enumerating objects: 335, done.
remote: Counting objects: 100% (53/53), done.
remote: Compressing objects: 100% (40/40), done.
remote: Total 335 (delta 19), reused 36 (delta 9), pack-reused 282
Receiving objects: 100% (335/335), 134.96 KiB | 2.59 MiB/s, done.
Resolving deltas: 100% (160/160), done.
» Cloned to /home/rbuckland/projects/github.com/inosion/madato
```

The default folder for repo clones is `P

### Smart Repository Inference

When you're in a subdirectory of your projects folder, `git-cloner` can infer the organization:

```bash
# From: ~/projects/github.com/myorg/
❯ git cloner my-repo
» Cloning https://github.com/myorg/my-repo → ~/projects/github.com/myorg/my-repo

# With tab completion!
❯ git cloner <TAB>
repo1  repo2  repo3  my-awesome-project  ...
```

## Manual Installation

1. Copy the binary to ~/.local/bin
2. Add the following alias to your `~/.gitconfig`

    ```
    [alias]
    cloner = !~/.local/bin/git-cloner
    ```

3. (Optional) Enable tab completion by sourcing the completion scripts:
   - Bash: `source support/git-cloner-completion.bash`
   - Zsh: `source support/git-cloner-completion.zsh`

## Details

`git-cloner` is a helper that takes a URL of a git repo, and clones it to your "project/workspace" directory, preserving the org/owner, or project name of the repository.

## Usage

### Simple repo name (when in org directory)
```bash
cd ~/projects/github.com/myorg
git cloner my-repo
# Clones: https://github.com/myorg/my-repo
```

### Org/repo format
```bash
git cloner torvalds/linux
# Clones: https://github.com/torvalds/linux → ~/projects/github.com/torvalds/linux
```

### Full URL
```bash
git cloner https://github.com/rust-lang/rust
# Clones: https://github.com/rust-lang/rust → ~/projects/github.com/rust-lang/rust
```

## Configuration

* the folder is configured via `env var CLONER_WORKSPACE`
   ```
   export CLONER_WORKSPACE=$HOME/workspace
   ```
* defaults to ~/projects
* Quickly cloning to a temporary folder:

    ```
    CLONER_WORKSPACE=/tmp/somefolder git cloner https://...
    ```

## Features


* Supports bitbucket style URLs (removes the leading `scm/`)
    ```
    » Cloning https://bitbucket.ihmc.us/scm/libs/log-tools.git → /home/username/bitbucket.ihmc.us/libs/log-tools
    Cloning into 'log-tools'...
    remote: Counting objects: 991, done.
    remote: Compressing objects: 100% (765/765), done.
    remote: Total 991 (delta 408), reused 0 (delta 0)
    Receiving objects: 100% (991/991), 102.26 KiB | 200.00 KiB/s, done.
    Resolving deltas: 100% (408/408), done.
    » Cloned to /home/username/bitbucket.ihmc.us/libs/log-tools
    ```

* Supports nested gitlab style URLs

    ```
    » Cloning https://gitlab.com/some-group/special/sub-project/tool-a.git → /home/username/gitlab.com/some-group/special/sub-project/tool-a
    Cloning into 'log-tools'...
    remote: Counting objects: 991, done.
    remote: Compressing objects: 100% (765/765), done.
    remote: Total 991 (delta 408), reused 0 (delta 0)
    Receiving objects: 100% (991/991), 102.26 KiB | 200.00 KiB/s, done.
    Resolving deltas: 100% (408/408), done.
    » Cloned to /home/username/gitlab.com/some-group/special/sub-project/tool-a
    ```

* Removes the .git extension if it is there

## CLI

```
git-cloner 0.1.0

USAGE:
    git-cloner [FLAGS] [OPTIONS] <url>

FLAGS:
    -d, --dry-run
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --workspace <workspace>     [env: CLONER_WORKSPACE=]  [default: ~/projects]

ARGS:
    <url>
```

## Appendix

This used to be written in python, but I converted it to rust so I did not have to manage a `py-venv` for a system wide "tool".