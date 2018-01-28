# rpomo [![Build Status](https://travis-ci.org/schumacher-m/rpomo.svg?branch=master)](https://travis-ci.org/schumacher-m/rpomo)

Pomodoro-esque CLI in Rust to be used in TMUX
* https://github.com/schumacher-m/rpomo-tmux-plugin

## Setup

* Rust
```
cargo build --release
cargo install --force
```

## Start
```
rpomo --start
```

## Stop
```
rpomo --stop
```

## Status
This will be used by the TMUX plugin to update your status and will either return:
`Working: 12:34` or `Break: 12:34` or `Idle`

```
rpomo --status
```
