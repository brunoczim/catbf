    ### jz begin
    # *(tape_start + tape_pos) ?= 0
    movb 0(%r12, %r14), %al
    testb %al, %al
    # *(tape_start + tape_pos) == 0
    jz .jz_label
    ### jz end
