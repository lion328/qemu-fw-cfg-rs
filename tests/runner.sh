#!/bin/sh

TARGET=$(echo $1 | sed 's/.*target\/\([^\/]*\).*/\1/')
ARCH=$(echo $TARGET | awk -F '-' '{ print $1 }')

if [ "$ARCH" = "i686" ]; then
    qemu-system-i386 \
        -kernel $1 \
        -m 32M \
        -display none \
        -fw_cfg opt/input.txt,file=tests/input.txt \
        -device isa-debug-exit \
        -serial stdio

    status=$(($? >> 1))
fi

if [ $status > 0 ]; then
    exit $status
fi
