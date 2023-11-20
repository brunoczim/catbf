    ### leave begin
.success:
    # return_0 = 0
    xorl %eax, %eax
.leave:
    # arg_0 = tape_start
    movq %r12, %rdi
    # catbf_detroy_tape(tape_start)
    call catbf_destroy_tape
    # restore registers
    popq %rbx
    popq %r12
    popq %r13
    popq %r14
    # return to caller
    ret
    ### leave end
