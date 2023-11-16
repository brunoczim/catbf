    ### put begin
    # arg_0 = interface
    movq %rbx, %rdi
    # arg_1 = *(tape_start + tape_pos)
    xorl %eax, %eax
    movb %al, 0(%r12, %r14)
    movl %esi, %eax
    # result_0 = fast_bfc_put(arg_0, arg_1)
    call fast_bfc_put
    # result_0 ?= result_0
    testb %al, %al
    # result0 < 0
    js .leave
    ### put end
