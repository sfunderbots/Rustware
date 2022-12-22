#!/bin/bash

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT_DIR="$CURR_DIR/../"

echo "Installing ER-Force simulator..."

if [[ "$OSTYPE" == 'darwin'* ]]; then
    echo "Installing dependencies with brew..."
    xcode-select --install
    # openssl isn't listed in the README instructions, but seems to be necessary for cmake
    brew install cmake git sdl2 protobuf libusb python@2 qt@5 openssl
else
    # https://github.com/robotics-erlangen/framework/blob/master/COMPILE.md
    echo "Installing dependencies with apt..."
    sudo apt install cmake protobuf-compiler libprotobuf-dev qtbase5-dev libqt5opengl5-dev g++ libusb-1.0-0-dev libsdl2-dev libqt5svg5-dev
fi

clone_dir="$PROJECT_ROOT_DIR/third_party/"
clone_name="erforce_simulator"
clone_path="$clone_dir/$clone_name"
if [ -d "$clone_path" ]; then
    echo "Removing old erforce sim..."
    rm -rf "$clone_path"
fi

cd $clone_dir
git clone https://github.com/sfunderbots/ER-Force-Simulator.git $clone_name
cd $clone_path

# Build the cli, our pybind version, and install the cli to the host computer
if [[ "$OSTYPE" == 'darwin'* ]]; then
    mkdir build-mac && cd build-mac
    # Need to manually point to openssl on mac for some reason
    # PIC is needed for pybind
    cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_POSITION_INDEPENDENT_CODE:BOOL=ON -DOPENSSL_ROOT_DIR=/usr/local/opt/openssl -DOPENSSL_LIBRARIES=/usr/local/opt/openssl/lib ..
    make -j$(nproc) simulator-cli
    sudo ln -sf $clone_path/build-mac/bin/simulator-cli.app/Contents/MacOS/simulator-cli /usr/local/bin/ersim
else
    mkdir build
    cd build
    # PIC is needed for pybind
    cmake -DCMAKE_BUILD_TYPE=Release -DCMAKE_POSITION_INDEPENDENT_CODE:BOOL=ON ..
    make -j$(nproc) simulator-cli
    sudo ln -sf $clone_path/build/bin/simulator-cli /usr/local/bin/ersim
fi
