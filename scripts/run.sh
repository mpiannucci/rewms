#!/bin/bash
set -e

if [[ ! -z "$@" ]]; then
    # If the user has given us a command, run it.
    $@
else
    # NGINX daemon
    nginx
    status=$?
    if [ $status -ne 0 ]; then
        echo "Failed to start NGINX: $status"
        exit $status
    fi

    # rewms application
    /usr/local/cargo/bin/rewms --port=9080 --wms-root="http://localhost:8080"
fi