#!/bin/sh

# Start nginx temporarily for certbot
nginx &

# Get certificate (will fail on first run if DNS not ready)
certbot --nginx -d permissionless.money -d www.permissionless.money --non-interactive --agree-tos --email your@email.com || true

# Stop temporary nginx
nginx -s stop

# Run nginx in foreground
nginx -g 'daemon off;'