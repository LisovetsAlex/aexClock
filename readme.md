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
shift + up, shift + down    - main window
q                           - quit
```

## How to compile
```
cargo build --release
sudo cp ~/ProgramCode/aexClock/target/release/aexClock /usr/local/bin/aexClock
aexClock
```