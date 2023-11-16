    .set TAPE_CHUNK_SIZE, 8192
    .text
    .extern catbf_grow_next
    .extern catbf_grow_prev
    .extern catbf_get
    .extern catbf_put
    .globl catbf_main

catbf_main:
