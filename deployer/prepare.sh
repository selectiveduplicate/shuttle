#!/usr/bin/env bash

##############################################################################################
# This file is run by Containerfile for extra preparation steps for this crate's final image #
##############################################################################################

# Patch crates to be on same versions
mkdir -p $CARGO_HOME
touch $CARGO_HOME/config.toml
if [[ $PROD != "true" ]]; then
    bash scripts/apply-patches.sh $CARGO_HOME/config.toml /usr/src/shuttle
fi

# Add the wasm32-wasi target for shuttle-next
rustup target add wasm32-wasi
# Add the wasm32 target for frontend frameworks
rustup target add wasm32-unknown-unknown

# Install common build tools for external crates
# The image should already have these: https://github.com/docker-library/buildpack-deps/blob/fdfe65ea0743aa735b4a5f27cac8e281e43508f5/debian/bookworm/Dockerfile
apt update
apt install -y llvm-dev libclang-dev clang cmake lld mold protobuf-compiler

# Binstall
curl -L --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/cargo-bins/cargo-binstall/main/install-from-binstall-release.sh | bash

# Common cargo build tools
cargo binstall -y --locked trunk@0.17.5

while getopts "p," o; do
case $o in
    "p") # if panamax is used, the '-p' parameter is passed
        # Make future crates requests to our own mirror
        # This is done after shuttle-next install in order to not sabotage it
        echo '
[source.shuttle-crates-io-mirror]
registry = "sparse+http://panamax:8080/index/"
[source.crates-io]
replace-with = "shuttle-crates-io-mirror"' >> $CARGO_HOME/config.toml
            ;;
        *)
            ;;
    esac
done
