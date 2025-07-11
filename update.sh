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
borders_on = true
border_color = "darkgray"
border_style = "rounded"

nav_selected_fg_color = "black"
nav_selected_bg_color = "cyan"
content_selected_color = "cyan"
bg_color = "black"
fg_color = "white"
scroll_color = "cyan"

bar_side_color = "magenta"
bar_filled_color = "cyan"
bar_empty_color = "blue"
bar_selected_side_color = "darkgray"
bar_selected_filled_color = "white"
bar_selected_empty_color = "gray"

[keybinds]
nav_up = "up"
nav_down = "down"

content_up = "shift+up"
content_down = "shift+down"
content_right = "shift+right"
content_left = "shift+left"

accept = "enter"
info = "tab"
cancel = "esc"
quit = "q"

EOF
else
    echo "Config already exists at $CONFIG_FILE"
fi

echo "Done."
