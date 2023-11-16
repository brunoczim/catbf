    .set TAPE_CHUNK_SIZE, 8192
    .text
    .extern fast_bfc_grow_next
    .extern fast_bfc_grow_prev
    .extern fast_bfc_get
    .extern fast_bfc_put
    .globl fast_bfc_main

fast_bfc_main:
