    ### leave begin
.success:
    # return_0 = 0
    xorl %eax, %eax
.leave:
    popq %r14
    popq %r13
    popq %r12
    popq %rbx
    ret
    ### leave end
