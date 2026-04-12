#!/bin/bash
set -e

# Inject SSH keys from environment variable
mkdir -p /root/.ssh
chmod 700 /root/.ssh

if [ -n "$GPUAAS_SSH_KEYS" ]; then
    echo "$GPUAAS_SSH_KEYS" > /root/.ssh/authorized_keys
    chmod 600 /root/.ssh/authorized_keys
    echo "SSH keys injected"
fi

# Start sshd in background
if command -v sshd >/dev/null 2>&1; then
    mkdir -p /run/sshd
    /usr/sbin/sshd -D &
    echo "sshd started"
fi

# Keep container alive
exec sleep infinity
