#compdef git-cloner

_git_cloner() {
  local -a repos
  repos=("${(@f)$(git-cloner complete 2>/dev/null)}")
  _describe 'repositories' repos
}

# For direct invocation: git-cloner <TAB>
compdef _git_cloner git-cloner

# For git subcommand: git cloner <TAB>
# This function is called by git's completion system
_git-cloner() {
  _git_cloner
}

# Register with git completion
compdef _git-cloner git-cloner