#!/bin/bash

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT_DIR="$CURR_DIR/../"

echo -e "\n\nInstalling Tigers AutoRef..."

cd "$PROJECT_ROOT_DIR/third_party"
clone_dir="tigers_autoref"
[ -d "$clone_dir"] && rm -r "$clone_dir"
git clone "https://github.com/TIGERs-Mannheim/AutoReferee.git" "$clone_dir"
cd "$clone_dir"
git checkout 5f473aaafd7c4c46bd34757172d10e35ade2fde8
./build.sh

echo -e "\n\nDone installing Tigers AutoRef"
