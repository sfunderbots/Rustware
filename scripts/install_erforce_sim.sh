#!/bin/bash

CURR_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT_DIR="$CURR_DIR/../"

echo "Installing ER-Force simulator..."

if [[ "$OSTYPE" == 'darwin'* ]]; then
    echo "Installing dependencies with brew..."
    brew install cmake git sdl2 protobuf libusb qt@5
else
    # https://github.com/robotics-erlangen/framework/blob/master/COMPILE.md
    echo "Installing dependencies with apt..."
    sudo apt install cmake protobuf-compiler libprotobuf-dev qtbase5-dev libqt5opengl5-dev g++ libusb-1.0-0-dev libsdl2-dev libqt5svg5-dev
fi

sim_location="/opt"
clone_name="erforce-framework"
sim_path="$sim_location/$clone_name"
if [ -d "$sim_path" ]; then
    echo "Removing old erforce sim..."
    sudo rm -r "$sim_path"
fi

cd $sim_location
sudo git clone https://github.com/robotics-erlangen/framework.git $clone_name
cd $sim_path
sudo mkdir build
cd build
sudo cmake ..
sudo make simulator-cli
if [[ "$OSTYPE" == 'darwin'* ]]; then
    sudo ln -sf $sim_path/build/bin/simulator-cli.app/Contents/MacOS/simulator-cli /usr/local/bin/ersim
else
    sudo ln -sf $sim_path/build/bin/simulator-cli /usr/local/bin/ersim
fi

