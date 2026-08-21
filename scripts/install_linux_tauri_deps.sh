#!/usr/bin/env bash
set -euo pipefail

sudo apt-get update
sudo apt-get install -y \
  build-essential \
  curl \
  file \
  libayatana-appindicator3-dev \
  libgdk-pixbuf2.0-dev \
  libglib2.0-dev \
  libgtk-3-dev \
  libjavascriptcoregtk-4.1-dev \
  librsvg2-dev \
  libsoup-3.0-dev \
  libssl-dev \
  libwebkit2gtk-4.1-dev \
  libxdo-dev \
  patchelf \
  pkg-config \
  wget
