#!/bin/bash

set -e

##This needs to match electron version
export npm_config_target=3.0.6
export npm_config_arch=x64
export npm_config_target_arch=x64
export npm_config_disturl=https://atom.io/download/electron
export npm_config_runtime=electron
export npm_config_build_from_source=true

mkdir -p ~/.electron-gyp
npm config set cache ~/.electron-gyp
yarn run neon build --release
