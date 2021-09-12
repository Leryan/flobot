cargo build --release
strip target/release/flobot
scp target/release/flobot srv.leila:/home/bot/flobot.upgrade
ssh srv.leila systemctl restart bot
