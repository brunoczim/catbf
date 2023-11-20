    ### leave begin
.success:
    # temp = 0
    xorb %r14b, %r14b
    jmp .leave
.failure:
    # temp = -1
    movb $-1, %r14b
.leave:
    # arg_0 = tape_start
    movq %r12, %rdi
    # catbf_detroy_tape(tape_start)
    call catbf_destroy_tape
    # return_0 = temp
    mov %r14b, %al
    # restore registers
    popq %rbx
    popq %r12
    popq %r13
    popq %r14
    # return to caller
    ret
    ### leave end
