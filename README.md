# FloBot

A terrible but friendly bot for Mattermost.

## Features

 * Blague: ask for a joke!
 * Edits: edit your message to replace with something else
 * SMS: send sms through octopush
 * Task: periodically run some stuff
 * Triggers: automatically answer to a list of words you manage
 * Werewolf: a simplistic version of the werewolf game. WIP.

Send `!help` on a channel where the bot is present, then `!help <module>`.

## Install & Use

 * Create a bot account with Admin privileges on Mattermost
 * Keep its access token safe
 * Add the bot to your team(s)
 * Create a debugging channel (preferably private) and invite the bot into

Create `flobot.env` from `flobot.env.example`.

```
apt install pkg-config libsqlite3-dev libssl-dev sqlite3

cargo build --release
```