#!/bin/bash

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT_DIR="$(git rev-parse --show-toplevel)"

echo "Installing pyenv..."
if command -v pyenv >/dev/null 2>&1 ; then
  echo "pyenv already installed"
elif [[ "$OSTYPE" == 'darwin'* ]]; then
  # I didn't seem to need the python build dependencies when trying
  # a fresh setup on MacOS
  curl https://pyenv.run | bash
  exec $SHELL
else
  # https://github.com/pyenv/pyenv/wiki#suggested-build-environment
  echo "Installing python build dependencies"
  sudo apt install make build-essential libssl-dev zlib1g-dev \
       libbz2-dev libreadline-dev libsqlite3-dev wget curl llvm \
       libncursesw5-dev xz-utils tk-dev libxml2-dev libxmlsec1-dev libffi-dev liblzma-dev
  # https://github.com/pyenv/pyenv-installer
  curl https://pyenv.run | bash
  exec $SHELL
fi

echo "Setting up pyenv environment..."
eval "$(pyenv init -)"
# Will prompt only if the version is already installed
echo "N" | PYTHON_CONFIGURE_OPTS="--enable-shared" pyenv install --force 3.9.7
pyenv virtualenv 3.9.7 rustware
pyenv activate rustware
pip3 install -r "$PROJECT_ROOT_DIR/requirements.txt"

