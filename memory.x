MEMORY
{
    FLASH : ORIGIN = 0x08000000, LENGTH = 1024K /* BANK_1_REGION_1 + BANK_1_REGION_2 + BANK_1_REGION_3 */
    RAM   : ORIGIN = 0x20000000, LENGTH =  320K /* RAM   : ORIGIN = 0x20010000, LENGTH =  320K */
    /* DTCM  ORIGIN = 0x20000000, LENGTH = 64K  (END 0x2000FFFF) */
    /* SRAM1 ORIGIN = 0x20010000, LENGTH = 240K (END 0x2004BFFF) */
    /* SRAM2 ORIGIN = 0x2004C000, LENGTH = 16K  (END 0x2004FFFF) */
    /* It _looks_ like GP-DMAs can access everything. DS10916 2.5, pg 18 */
    OTP   : ORIGIN = 0x1ff0f000, LENGTH =  512
}

/* This is where the call stack will be allocated. */
/* The stack is of the full descending type. */
/* NOTE Do NOT modify `_stack_start` unless you know what you are doing */
_stack_start = ORIGIN(RAM) + LENGTH(RAM);