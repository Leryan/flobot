#!/bin/bash

echo "Flobot upgrade."

if [ ! "$(id -u)" = 0 ]; then
	echo "Not root, cannot perform upgrade."
	exit 0
fi

if [ -f flobot.upgrade ]; then
	echo "Upgrade found, moving files"
	md5sum flobot.upgrade flobot
	mv flobot.upgrade flobot
	chmod +x flobot
else
	echo "No upgrade found."
fi

chown $1:$1 flobot
