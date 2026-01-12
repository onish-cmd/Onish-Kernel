.section .text._start
.global _start

_start:
    csrr a0, mhartid
    bnez a0, park

    li t0, 0x10000000
    li t1, 79
    sb t1, 0(t0)

    la sp, __stack_top

    la t0, __bss_start
    la t1, __bss_end
bss_clear:
    bge t0, t1, launch
    sd zero, 0(t0)
    addi t0, t0, 8
    j bss_clear

launch:
    tail kmain

park:
    wfi
    j park

.section .bss.stack
.align 16
stack_base:
    .space 4096 * 4
__stack_top: