#!/bin/bash

# This script downloads and compiles the proto files for all
# relevant SSL software

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT_DIR="$(git rev-parse --show-toplevel)"

PROTO_BASE_DIR="$PROJECT_ROOT_DIR/third_party"

brew install gnu-sed

# The unix sed that comes with MacOS behaves slightly differently than
# gnu sed. Use gnu sed for consistency.
sed () {
  if [[ "$OSTYPE" == 'darwin'* ]]; then
    gsed "$@"
  else
    command sed "$@"
  fi
}

# Add a package definition to the start of the given .proto files
prepend_proto_package() {
  package_name="$1"
  shift
  arr=("$@")
  for f in "${arr[@]}"; do
    # Add the package right after the syntax definition
    sed -i "/^syntax.*=.*proto.*/a package $package_name;" "$f"
  done
}

# Adjusts the import paths in the generated proto python code to be relative to the
# given prefix
make_proto_python_imports_relative_to_root() {
  current_path="$1"
  shift
  new_path="$1"
  shift
  arr=("$@")
  for f in "${arr[@]}"; do
    sed -i -e "s/from $current_path import/from $new_path import/g" "$f"
  done
}

# Appends a prefix to import statements in the .proto files
make_proto_imports_relative_to_root() {
  prefix="$1"
  shift
  arr=("$@")
  for f in "${arr[@]}"; do
    sed -i -e "s/import \"/import \"$prefix\//g" "$f"
    # Don't add a prefix for any google imports
    sed -i -e "s/import \"$prefix\/google/import \"google/g" "$f"
  done
}

install_protos() {
  local name="$1"
  local repo_proto_path="$2"
  local name_snake_case="${name//-/_}"
  local install_dir="$PROTO_BASE_DIR/$name_snake_case"
  mkdir -p "$install_dir"

  cd /tmp
  [ -d "/tmp/$name" ] && rm -rf "/tmp/$name"
  git clone "https://github.com/RoboCup-SSL/$name.git"
  cp "$name/$repo_proto_path/"*.proto "$install_dir/"
  rm "$install_dir"/*_legacy.proto || echo "No legacy protos to remove"

  # We need to add packages to each repo because some messages have the same name
  prepend_proto_package "$name_snake_case" "$install_dir/"*.proto
}

echo "Installing SSL protobufs..."
install_protos ssl-simulation-protocol proto
install_protos ssl-game-controller proto
install_protos ssl-vision src/shared/proto
echo "Done installing SSL protobufs"
