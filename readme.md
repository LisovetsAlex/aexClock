# Super Clock
Terminal UI app with a big clock, ability to connect to internet...

Requires nmcli to run.

![Preview](preview.png)

## How to start
```
cargo run
```

## How to navigate
```
up, down                    - nav menu
q                           - quit

Internet
--------
shift + up, shift + down    - scroll list
tab                         - connection info
enter                       - prompt password to connect to wifi
esc                         - cancel prompt password
```

## How to compile
Will create and compile new version of this app
```
./update.sh
```

## Create config file at ~/.config/aex/clock.toml

```
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

```