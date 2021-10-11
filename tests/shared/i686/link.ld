SECTIONS {
    . = 1M;
    .text : {
        * (.multiboot);
        * (.text.*);
    }
    .data ALIGN(4K) : {
        * (.data.*);
    }
    .rodata ALIGN(4K) : {
        * (.rodata.*);
    }
    .bss ALIGN(4K) : {
        * (.bss.*);
        *(COMMON);
    }
}
