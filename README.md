# FloBot

A terrible but friendly bot for Mattermost.

## Features

### Triggers

```
!trigger list
!trigger text "some text" "triggers this response"
!trigger reaction "some text" :triggered_emoji:
!trigger del "some text"
```

### Edits

```
!edits list
!edits add "some text" "will be replaced with this one"
!edits del "some text"
!e some text
```

## Install & Use

 * Create a bot account with Admin privileges on Mattermost
 * Keep it's access token safe
 * Add the bot to your team(s)
 * Create a debugging channel (preferably private) and invite the bot into

```
cargo build --release
```

Copy `target/release/flobot` and `migrations` folder.

Create `flobot.env` from `flobot.env.example`.

## Diesel

```
# Bootstrap dev from scratch

apt install pkg-config libsqlite3-dev libssl-dev sqlite3

cargo install diesel_cli --no-default-features --features sqlite

diesel setup
```

```
# regular use

diesel migration run

# check that migrations works correctly -> wipes out data
diesel migration redo
```
