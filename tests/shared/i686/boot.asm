.equ MULTIBOOT_MAGIC, 0x1BADB002
.equ MULTIBOOT_FLAGS, 1
.equ MULTIBOOT_CHECKSUM, -(MULTIBOOT_MAGIC + MULTIBOOT_FLAGS)

.section .multiboot

.align 4
.int MULTIBOOT_MAGIC
.int MULTIBOOT_FLAGS
.int MULTIBOOT_CHECKSUM

.text
.global _start

_start:
    mov esp, OFFSET stack_top
    call main
    push 0
    call exit

.bss

.align 4096
.skip 128 * 1024
stack_top:
