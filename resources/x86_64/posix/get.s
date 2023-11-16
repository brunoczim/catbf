    ### get begin
    # start =? end
    cmpq %r13, %r14
    # start == end
    jne .get_growed_next
    # arg_0 = tape_start
    movq %r12, %rdi
    # arg_1 = tape_len
    movq %r13, %rsi
    # return_0 = fast_bfc_grow_next(arg_0, arg_1)
    call fast_bfc_grow_next
    # tape_start = return_0
    movq %rax, %r12
    # tape_len += 8192
    addq $8192, %r13
.get_growed_next:
    # arg_0 = interface
    movq %rbx, %rdi
    # return_0 = fast_bfc_get(arg_0)
    call fast_bfc_get
    # return_0 ?= return_0
    testw %ax, %ax
    # return_0 < 0
    js .leave
    # bswap return_0
    ror $8, %ax
    # *(tape_start + tape_pos) = return_0
    movw %ax, 0(%r12, %r14)
    ### get end
