    ### enter begin
    # save registers
    # tape_pos: u64
    pushq %r14
    # tape_len: u64
    pushq %r13
    # tape_start: *mut u8
    pushq %r12
    # interface: *mut interface
    pushq %rbx
    # interface = arg_0
    movq %rdi, %rbx
    # tape_pos = 0
    xorq %r14, %r14
    # result_0 = catbf_create_tape()
    call catbf_create_tape
    # tape_len = TAPE_CHUNK_SIZE
    movq $TAPE_CHUNK_SIZE, %r13
    # tape_start = result_0
    movq %rax, %r12
    ### enter end
