#!/bin/bash
systemctl stop exampleauth;
cd /srv/http/vhosts/rocket-sqlx-authentication-api/;
git stash;
git stash drop;
git pull origin master;
cargo build;
systemctl start exampleauth;
systemctl status exampleauth;
#journalctl -f -u exampleauth;
