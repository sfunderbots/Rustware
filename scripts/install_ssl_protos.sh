#!/bin/bash

# This script downloads and compiles the proto files for all
# relevant SSL software

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT_DIR="$(git rev-parse --show-toplevel)"

PROTO_BASE_DIR="$PROJECT_ROOT_DIR/third_party"

if [[ "$OSTYPE" == 'darwin'* ]]; then
    brew install gnu-sed
fi

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
  # Python proto does not play nicely when .proto files have the same name. In order for the compiler
  # to treat two files with the same name (but different paths/modules) as different files when
  # generating proto, they must have separate paths and the protoc proto_path must be set to the
  # shared root directory of these files. This will result in the generated proto message descriptors
  # including the relative paths in their names, and prevent any conflicts when both messages
  # are imported in python. Otherwise you will end up with errors such as:
  # "messages.proto: A file with this name is already in the pool."
  # See: https://github.com/protocolbuffers/protobuf/issues/3002
  make_proto_imports_relative_to_root "$name_snake_case" "$install_dir"/*.proto

  # Add wrappers to contain multiple messages. Used for sending batches of raw data to the GUI
  if [ "$name_snake_case" = "ssl_vision" ]; then
    cat >> "$install_dir/messages_robocup_ssl_wrapper.proto" <<EOL
message SSL_WrapperPackets {
  repeated SSL_WrapperPacket packets = 1;
}
EOL
  elif [ "$name_snake_case" = "ssl_game_controller" ]; then
    cat >> "$install_dir/ssl_gc_referee_message.proto" <<EOL
message Referees {
  repeated Referee packets = 1;
}
EOL
  fi

  protoc --python_out="$PROTO_BASE_DIR/" --proto_path="$PROTO_BASE_DIR/" "$install_dir"/*.proto
  # The python imports in the generated proto files are assumed to be relative to the protoc proto_path,
  # which in our case is not the project root module/directory. We have to manually adjust the import
  # paths to be relative to the project root.
  # Eg. "from ssl_vision import *" becomes "from third_party.ssl_vision import *"
  # See: https://github.com/protocolbuffers/protobuf/issues/881
  make_proto_python_imports_relative_to_root "$name_snake_case" "third_party.$name_snake_case" "$install_dir"/*_pb2.py
}

echo "Installing SSL protobufs..."
install_protos ssl-simulation-protocol proto
install_protos ssl-game-controller proto
install_protos ssl-vision src/shared/proto

echo "Done installing SSL protobufs"
