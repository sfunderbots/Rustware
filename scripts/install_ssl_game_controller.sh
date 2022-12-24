#!/bin/bash

# This script downloads and compiles the proto files for all
# relevant SSL software

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT_DIR="$(git rev-parse --show-toplevel)"

# MacOS doesn't come with wget
if [[ "$OSTYPE" == 'darwin'* ]]; then
    brew install wget
    # https://github.com/RoboCup-SSL/ssl-game-controller/releases
    BINARY_URL="https://github.com/RoboCup-SSL/ssl-game-controller/releases/download/v2.16.1/ssl-game-controller_v2.16.1_darwin_amd64"
else
    # https://github.com/RoboCup-SSL/ssl-game-controller/releases
    BINARY_URL="https://github.com/RoboCup-SSL/ssl-game-controller/releases/download/v2.16.1/ssl-game-controller_v2.16.1_linux_amd64"
fi


FILENAME="$PROJECT_ROOT_DIR/third_party/ssl_game_controller/binary"
wget "$BINARY_URL" -O "$FILENAME"
chmod +x "$FILENAME"
