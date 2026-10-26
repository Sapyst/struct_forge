#!/usr/bin/env bash
# StructForge: Cyberspace Substrate Launcher
cd "$(dirname "$0")"

if [ ! -f "target/release/struct_forge" ]; then
    echo "Compilando StructForge em modo release..."
    cargo build --release
fi

./target/release/struct_forge
