    ### enter begin
    # interface: *mut interface
    pushq %rbx
    # tape_start: *mut u8
    pushq %r12
    # tape_len: u64
    pushq %r13
    # tape_pos: u64
    pushq %r14
    # interface = arg_0
    movq %rdi, %rbx
    # tape_start = arg_1
    movq %rsi, %r12
    # tape_len = arg_2
    movq %rdx, %r13
    # tape_pos = 0
    xorq %r14, %r14
    ### enter end
