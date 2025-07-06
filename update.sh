#!/bin/bash

set -e

CARGO_TOML="Cargo.toml"
BIN_PATH="/usr/local/bin/aexClock"
BACKUP_DIR="/usr/local/bin"
CONFIG_DIR="$HOME/.config/aex"
CONFIG_FILE="$CONFIG_DIR/clock.toml"

# Step 1: Bump version
current_version=$(grep '^version =' "$CARGO_TOML" | head -1 | cut -d '"' -f2)
echo "Current version: $current_version"

IFS='.' read -r major minor patch <<< "$current_version"
minor=$((minor + 1))
patch=0
new_version="$major.$minor.$patch"
echo "New version: $new_version"

sed -i.bak -E "s/^version = \".*\"/version = \"$new_version\"/" "$CARGO_TOML"

# Step 2: Backup binary if exists
if [ -f "$BIN_PATH" ]; then
    echo "Backing up existing binary..."
    sudo mv "$BIN_PATH" "$BACKUP_DIR/aexClock_$current_version"
    echo "Renamed $BIN_PATH to $BACKUP_DIR/aexClock_$current_version"
fi

# Step 3: Build and install
cargo build --release
sudo cp target/release/aexClock "$BIN_PATH"
echo "Installed aexClock to $BIN_PATH"

# Step 4: Create config if not present
if [ ! -f "$CONFIG_FILE" ]; then
    echo "Creating default config at $CONFIG_FILE"
    mkdir -p "$CONFIG_DIR"
    cat > "$CONFIG_FILE" <<EOF
[themes]
border_color = "white"
border_style = "rounded"
nav_selected_color = "white"
content_selected_color = "cyan"
bg_color = "black"
fg_color = "white"
scroll_color = "cyan"
borders_on = true

[keybinds]
nav_up = "up"
nav_down = "down"
content_up = "shift+up"
content_down = "shift+down"
accept = "enter"
info = "tab"
cancel = "esc"
quit = "q"
EOF
else
    echo "Config already exists at $CONFIG_FILE"
fi

echo "Done."
