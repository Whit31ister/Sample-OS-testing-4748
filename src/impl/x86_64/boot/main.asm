

global start
extern rust_kernel_entry


section .text
bits 64  ;for changing bit change this first
start:
    ; print

    mov dword [0xb8000], 0x2f4b2f4f

    call rust_kernel_entry

.hang:
    hlt
    jmp .hang
