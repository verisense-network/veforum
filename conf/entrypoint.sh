#!/usr/bin/env sh

envsubst '$MEILI_MASTER_KEY' < /tmp/nginx.conf > /etc/nginx/nginx.conf
exec nginx -g 'daemon off;'
