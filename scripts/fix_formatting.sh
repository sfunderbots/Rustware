#!/bin/bash

###########################################################
# Automatically format code <3
###########################################################

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
REPO_ROOT_DIR="$(git rev-parse --show-toplevel)"

function run_python_formatting () {
    black "$REPO_ROOT_DIR/gui/"
}

function run_git_diff_check(){
    printf "Checking for merge conflict markers...\n\n"
    cd $CURR_DIR && git -c "core.whitespace=-trailing-space" --no-pager diff --check
    if [[ "$?" != 0 ]]; then
        printf "***Please fix merge conflict markers!***\n\n"
        exit 1
    fi
}

function run_eof_new_line(){
    printf "Adding missing new lines to end of files...\n\n"

    # adds missing new lines to the end of non-binary files
    cd $CURR_DIR/../ && git grep -zIl '' | while IFS= read -rd '' f; do tail -c1 < "$f" | read -r _ || echo >> "$f"; done
    if [[ "$?" != 0 ]]; then
        printf "***Failed to add missing new lines!***\n\n"
        exit 1
    fi
}

function run_rust_formatting() {
    cargo fmt --manifest-path="$REPO_ROOT_DIR/ai/Cargo.toml"

}

run_git_diff_check
run_eof_new_line
run_python_formatting
run_rust_formatting
