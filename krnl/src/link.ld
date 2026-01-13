/* link.ld */
OUTPUT_ARCH( "riscv" )
ENTRY( _start )

SECTIONS
{
  . = 0x80200000;

  .text : {
    KEEP(*(.text._start))
    *(.text .text.*)
  }

  .rodata : ALIGN(4K) {
    *(.rodata .rodata.*)
  }

  .data : ALIGN(4K) {
    *(.data .data.*)
  }

  .bss : ALIGN(4K) {
    PROVIDE(__bss_start = .); 
    *(.bss .bss.*)
    *(COMMON)
    PROVIDE(__bss_end = .);
  }

  /* Move the stack to a safe distance or define it clearly */
  . = ALIGN(16);
  . += 0x8000; /* 32KB Stack */
  PROVIDE(__stack_top = .);
}
