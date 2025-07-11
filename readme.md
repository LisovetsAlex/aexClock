# Super Clock
Terminal UI app for linux with a big clock, ability to connect to internet, control volume of different apps...

Requires nmcli, pactl to run.

![Preview](preview.png)

## How to start
```
cargo run
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

## How to compile
Will create and compile new version of this app
```
./update.sh
```

Then start with `aexClock`