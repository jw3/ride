#!/usr/bin/env bash

auth=()
if [[ "$1" == "--with-auth" ]]; then
  auth=("-c" "/myuser/mosquitto.conf")
fi

docker run --name mqtt --rm -p 1883:1883 quay.io/kboone/mosquitto-ephemeral:latest mosquitto -v "${auth[@]}"
