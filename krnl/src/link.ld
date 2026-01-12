/* link.ld */
OUTPUT_ARCH( "riscv" )
ENTRY( _start )

SECTIONS
{
  /* QEMU Virt RAM starts here */
  . = 0x80000000;

  /* 1. Code Section */
  .text : {
    KEEP(*(.text._start))
    *(.text .text.*)
  }

  /* 2. Read-only Data (Strings) */
  .rodata : {
    *(.rodata .rodata.*)
  }

  /* 3. Initialized Data */
  .data : {
    *(.data .data.*)
  }

  /* 4. Uninitialized Data (BSS) */
  .bss : {
    __bss_start = .;  /* Mark the start for boot.S to clear */
    *(.bss .bss.*)
    *(COMMON)
    __bss_end = .;    /* Mark the end */
  }

  /* 5. The Stack */
  . = ALIGN(16);      /* RISC-V 64-bit stack MUST be 16-byte aligned */
  . += 0x4000;        /* Allocate 16KB for the stack */
  __stack_top = .;    /* The address our boot.S loads into 'sp' */
}