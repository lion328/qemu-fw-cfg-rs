#!/bin/bash

TARGET=$(echo $1 | sed 's/.*target\/\([^\/]*\).*/\1/')
ARCH=$(echo $TARGET | awk -F '-' '{ print $1 }')

QEMU_OPTS=" -m 32M"
QEMU_OPTS+=" -display none"
QEMU_OPTS+=" -fw_cfg opt/input.txt,file=tests/input.txt"
QEMU_OPTS+=" -fw_cfg opt/567890123456789012345678901234567890123456789012345,file=tests/input.txt"
QEMU_OPTS+=" -serial stdio"

if [[ "$ARCH" == i686 ]]; then
    qemu-system-i386 $QEMU_OPTS \
        -device isa-debug-exit \
        -kernel "$@"
elif [[ "$ARCH" == riscv32* ]]; then
    if [[ -z "$GDB" ]]; then
        qemu-system-riscv32 $QEMU_OPTS \
            -machine virt \
            -bios none \
            -kernel "$@"
    else
        riscv32-elf-gdb -ex 'target remote :1234' "$@"
    fi
else
    echo Unsupported TARGET=$TARGET
fi

status=$(($? >> 1))

if [ $status -gt 0 ]; then
    exit $status
fi
