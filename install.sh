#!/bin/sh
# One-line install: curl -fsSL https://raw.githubusercontent.com/Sanidhya14321/clipkit-studio/main/install.sh | sh
# Downloads the release for this machine, checks its checksum, installs to ~/.clipkit and
# offers `ck init`. CK_RELEASE_BASE overrides the download folder; CK_HOME the install folder.
set -eu
repo="Sanidhya14321/clipkit-studio"
base="${CK_RELEASE_BASE:-https://github.com/$repo/releases/latest/download}"
home_dir="${CK_HOME:-$HOME/.clipkit}"

os=$(uname -s | tr '[:upper:]' '[:lower:]')
arch=$(uname -m)
[ "$arch" = aarch64 ] && arch=arm64
case "$os-$arch" in
  darwin-arm64 | linux-x86_64) ;;
  *) echo "No ClipKit release for $os-$arch. Build from source: see the README." >&2; exit 1 ;;
esac
name="clipkit-$os-$arch"

tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
echo "Downloading $name"
curl -fsSL "$base/$name.tar.gz" -o "$tmp/$name.tar.gz"
curl -fsSL "$base/$name.tar.gz.sha256" -o "$tmp/$name.tar.gz.sha256"
if command -v shasum >/dev/null 2>&1; then
  (cd "$tmp" && shasum -a 256 -c "$name.tar.gz.sha256" >/dev/null)
else
  (cd "$tmp" && sha256sum -c "$name.tar.gz.sha256" >/dev/null)
fi
tar -xzf "$tmp/$name.tar.gz" -C "$tmp"
mkdir -p "$home_dir"
rm -rf "$home_dir/bin" "$home_dir/render-worker" "$home_dir/assets"
mv "$tmp/$name/bin" "$tmp/$name/render-worker" "$tmp/$name/assets" "$home_dir/"
echo "Installed to $home_dir/bin/ck"

for tool in ffmpeg ffprobe node git; do
  command -v "$tool" >/dev/null 2>&1 || echo "Missing $tool: install it before using ClipKit."
done
case ":$PATH:" in
  *":$home_dir/bin:"*) ;;
  *) echo "Add to your shell profile: export PATH=\"$home_dir/bin:\$PATH\"" ;;
esac
# `ck init` shows what it will write and asks first, so it needs the terminal.
if [ -z "${CK_NO_INIT:-}" ] && [ -r /dev/tty ] && [ -w /dev/tty ]; then
  "$home_dir/bin/ck" init </dev/tty >/dev/tty 2>&1 || echo "ck init did not finish; run it again later."
fi
