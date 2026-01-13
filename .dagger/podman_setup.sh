#!/bin/bash
# Creates a podman machine for use with dagger
PODMAN_MACHINE="${1:-podman-machine-default}" 
podman machine inspect podman-machine-default > /dev/null 2>&1
if [ $? -ne 0 ]; then
    echo "* Creating $PODMAN_MACHINE"
    podman machine init -m 10240 --now $PODMAN_MACHINE
fi

echo "* Configuring $PODMAN_MACHINE"
podman machine ssh $PODMAN_MACHINE sudo modprobe iptable_nat
podman machine ssh $PODMAN_MACHINE sudo setenforce Permissive
