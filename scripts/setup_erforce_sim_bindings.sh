#!/bin/bash

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT_DIR="$CURR_DIR/../"

echo "Installing Custom ER-Force simulator with python bindings..."

if [[ "$OSTYPE" == 'darwin'* ]]; then
    echo "Installing dependencies with brew..."
    brew install cmake git sdl2 protobuf libusb qt@5
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
    sudo rm -r "$clone_path"
fi

cd $clone_dir
git clone https://github.com/sfunderbots/ER-Force-Simulator.git $clone_name
cd $clone_path
mkdir build
cd build
cmake -DCMAKE_POSITION_INDEPENDENT_CODE:BOOL=ON ..
make erforce_simulator -j$(nproc)
#if [[ "$OSTYPE" == 'darwin'* ]]; then
#    sudo ln -sf $sim_path/build/bin/simulator-cli.app/Contents/MacOS/simulator-cli /usr/local/bin/ersim
#else
#    sudo ln -sf $sim_path/build/bin/simulator-cli /usr/local/bin/ersim
#fi

