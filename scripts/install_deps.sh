#!/bin/bash

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT_DIR="$(git rev-parse --show-toplevel)"

echo "Installing project dependencies..."

if [[ "$OSTYPE" == 'darwin'* ]]; then
    echo "MacOS detected. Running brew update..."
    brew update
else
    echo "Linux detected. Running apt update..."
    sudo apt-get update
fi

echo "Installing package dependencies..."
if [[ "$OSTYPE" == 'darwin'* ]]; then
    brew install protobuf 
else
    sudo apt install protobuf-compiler
fi

./install_ssl_protos.sh
./install_ssl_game_controller.sh
./install_erforce_sim.sh

echo "Done installing all dependencies!"
