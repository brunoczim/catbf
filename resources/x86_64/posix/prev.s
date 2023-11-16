    ### prev begin
    # start &? start
    testq %r14, %r14
    # start != 0
    jnz .growed_prev
    # arg_0 = tape_start
    movq %r12, %rdi
    # arg_1 = tape_len
    movq %r13, %rsi
    # return_0 = fast_bfc_grow_next(arg_0, arg_1)
    call fast_bfc_grow_prev
    # tape_pos += 8192
    addq $8192, %r14
    # tape_start = return_0
    movq %rax, %r12
    # tape_len += 8192
    addq $8192, %r13
.growed_prev:
    # tape_pos -= 1
    decq %r14
    ### prev end
