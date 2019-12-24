#!/bin/sh

pushd kaze; cargo test; popd
pushd kaze-sim-tests; cargo test; popd
