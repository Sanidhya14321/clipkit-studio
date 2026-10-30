#!/usr/bin/env bash
# Builds dist/clipkit-<os>-<arch>.tar.gz and its .sha256. Expects `just studio-build` and
# `just worker` to have run: the binary embeds studio/dist, and the worker ships beside it.
set -euo pipefail
os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
[ "$arch" = aarch64 ] && arch=arm64
name="clipkit-$os-$arch"
[ -f studio/dist/index.html ] || { echo "run just studio-build first" >&2; exit 1; }
[ -f render-worker/bundle/index.html ] || { echo "run just worker first" >&2; exit 1; }
cargo build --release -p ck-cli
stage="dist/$name"
rm -rf "$stage"
mkdir -p "$stage/bin" "$stage/render-worker"
cp target/release/ck "$stage/bin/ck"
cp -R render-worker/src render-worker/bundle render-worker/node_modules render-worker/package.json "$stage/render-worker/"
mkdir -p "$stage/assets"
cp -R assets/sfx-elements-audio assets/duru-original-music "$stage/assets/"
tar -czf "dist/$name.tar.gz" -C dist "$name"
(cd dist && shasum -a 256 "$name.tar.gz" >"$name.tar.gz.sha256")
echo "dist/$name.tar.gz"
