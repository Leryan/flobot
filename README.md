# FloBot

 * Uses Sqlite3

## Diesel

```
# Bootstrap dev from scratch

apt install libsqlite3-dev libssl-dev sqlite3

cargo install diesel_cli --no-default-features --features sqlite

diesel setup
```

```
# regular use

diesel migration run

# check that migrations works correctly -> wipes out data
diesel migration redo
```
