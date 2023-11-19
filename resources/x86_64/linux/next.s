    ### next begin
    # start =? end
    cmpq %r13, %r14
    # start == end
    jne .growed_next
    # arg_0 = tape_start
    movq %r12, %rdi
    # arg_1 = tape_len
    movq %r13, %rsi
    # return_0 = catbf_grow_next(arg_0, arg_1)
    call catbf_grow_next
    # result_0 ?= null
    test %rax, %rax
    # result == null
    jz .leave
    # tape_start = return_0
    movq %rax, %r12
    # tape_len += TAPE_CHUNK_SIZE
    addq $TAPE_CHUNK_SIZE, %r13
.growed_next:
    # tape_pos += 1
    incq %r14
    ### next end
