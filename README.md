# git-cloner

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

1. Copy the binary to ~/.local/bin
2. Add the following alias to your `~/.gitconfig`

    ```
    [alias]
    cloner = !~/.local/bin/git-cloner
    ```

## Details

`git-cloner` is a helper that takes a URL of a git repo, and clones it to your "project/workspace" directory, preserving the org/owner, or project name of the repository.

* the folder is configured via `env var CLONER_WORKSPACE`
   ```
   export CLONER_WORKSPACE=$HOME/workspace
   ```
* defaults to ~/projects
* Quckly cloning to a temporary folder is 
  
    ```
    CLONER_WORKSPACE=/tmp/somefolder git cloner http://....
    ```


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