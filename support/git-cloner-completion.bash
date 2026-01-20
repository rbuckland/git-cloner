# git-cloner-completion.bash

_git_cloner_complete()
{
    local cur prev
    COMPREPLY=()
    cur="${COMP_WORDS[COMP_CWORD]}"
    prev="${COMP_WORDS[COMP_CWORD-1]}"

    # Call git-cloner complete and use its output for completion
    local repos
    repos=$(git-cloner complete 2>/dev/null)
    COMPREPLY=( $(compgen -W "${repos}" -- "$cur") )
    return 0
}

complete -F _git_cloner_complete git-cloner

# Wrapper function for git alias
_git_cloner() {
    local cur
    _get_comp_words_by_ref -n =: cur

    # Call git-cloner complete and use its output for completion
    local repos
    repos=$(git-cloner complete 2>/dev/null)
    COMPREPLY=( $(compgen -W "${repos}" -- "$cur") )
    __ltrim_colon_completions "$cur"
}

# For git subcommand: git cloner
# This requires git's completion system to be loaded.
if declare -f __git_complete &>/dev/null; then
    __git_complete cloner _git_cloner
fi