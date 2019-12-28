#!/usr/bin/env sh

su speculo -c "/usr/local/bin/create-keys.sh"
touch /var/log/cron.log
chown speculo:speculo /var/log/cron.log
cron && tail -f /var/log/cron.log 
