#!/bin/bash

mkdir -p sources
cd sources

# Clone the repositories
if [ ! -d "SteelMC" ]; then
    # Copy local SteelMC instead of cloning (since it's a local project)
    cp -r ../../SteelMC .
fi

if [ ! -d "yarn" ]; then
    git clone https://github.com/FabricMC/yarn.git
fi

cd yarn
# Decompile the yarn mappings
./gradlew decompileVineFlower

cd ../..

cargo run --release
