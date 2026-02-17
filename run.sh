#!/bin/bash

mkdir -p sources
cd sources

# Clone the repositories
if [ ! -d "SteelMC" ]; then
    git clone https://github.com/4lve/SteelMC.git
fi

# Generate Steel's registration files (classes.json -> generated blocks.rs/items.rs)
if [ ! -f "SteelMC/steel-core/src/behavior/generated/blocks.rs" ]; then
    echo "Building SteelMC to generate registration files..."
    (cd SteelMC && cargo build -p steel-core 2>&1 || true)
fi

if [ ! -d "yarn" ]; then
    git clone https://github.com/FabricMC/yarn.git
fi

cd yarn
# Decompile the yarn mappings
./gradlew decompileVineFlower

cd ../..

cargo run --release
