#!/bin/sh
# Runs whatsup-desktop from the Debian podman container on the host desktop, to
# test the Debian build (above all the libadwaita 1.7 shortcuts fallback) on
# real libraries. Build the image first:
#
#   podman build -t whatsup-desktop-deb packaging/debian
#
# then run this from the repo root:  packaging/debian/run.sh
#
# What it does and doesn't cover:
#  - The window, Settings and the Keyboard Shortcuts dialog (Ctrl+?) run on
#    Debian's libadwaita 1.7, which is the point: that is the one code path
#    this build adds, and it needs no GPU or web content.
#  - Rendering is software (llvmpipe), so this is NOT a test of the NVIDIA
#    graphics paths; test those on Fedora (same driver) or a real Debian box.
#  - WebKit's sandbox may not start inside the container; if WhatsApp Web
#    doesn't load, the shortcuts UI still does, which is what to check here.
#
# Account data persists in target/debian-run-home so a QR scan is not needed
# every run.
set -eu

: "${WAYLAND_DISPLAY:?set WAYLAND_DISPLAY (a Wayland session is required)}"
: "${XDG_RUNTIME_DIR:?XDG_RUNTIME_DIR is not set}"

root=$(cd "$(dirname "$0")/../.." && pwd)
home="$root/target/debian-run-home"
mkdir -p "$home"

# Build the binary the Debian way (1.7 fallback) inside the container.
podman run --rm -v "$root":/src:Z -w /src whatsup-desktop-deb \
    cargo build --release --locked --no-default-features

exec podman run --rm -it \
    --security-opt seccomp=unconfined \
    --security-opt label=disable \
    --device /dev/dri \
    -v "$root":/src -w /src \
    -v "$XDG_RUNTIME_DIR/$WAYLAND_DISPLAY":/run/wayland/"$WAYLAND_DISPLAY" \
    -v "$home":/home/app \
    -e HOME=/home/app \
    -e XDG_RUNTIME_DIR=/run/wayland \
    -e WAYLAND_DISPLAY="$WAYLAND_DISPLAY" \
    -e GDK_BACKEND=wayland \
    -e LIBGL_ALWAYS_SOFTWARE=1 \
    whatsup-desktop-deb \
    dbus-run-session target/release/whatsup-desktop
