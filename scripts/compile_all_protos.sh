#!/bin/bash

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT_DIR="$(git rev-parse --show-toplevel)"

compile_proto_file() {
  protoc --proto_path="$PROJECT_ROOT_DIR" --python_out="$PROJECT_ROOT_DIR" "$1"
}

config_protos=$(find $PROJECT_ROOT_DIR/config/ -type f -name "*.proto")
for f in $config_protos; do
    echo $f
    compile_proto_file "$f"
done
