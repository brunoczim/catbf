    ### prev begin
    # start &? start
    testq %r14, %r14
    # start != 0
    jnz .growed_prev
    # arg_0 = tape_start
    movq %r12, %rdi
    # arg_1 = tape_len
    movq %r13, %rsi
    # return_0 = catbf_grow_prev(arg_0, arg_1)
    call catbf_grow_prev
    # result_0 ?= null
    test %rax, %rax
    # result == null
    jz .leave
    # tape_pos += TAPE_CHUNK_SIZE
    addq $TAPE_CHUNK_SIZE, %r14
    # tape_start = return_0
    movq %rax, %r12
    # tape_len += TAPE_CHUNK_SIZE
    addq $TAPE_CHUNK_SIZE, %r13
.growed_prev:
    # tape_pos -= 1
    decq %r14
    ### prev end
