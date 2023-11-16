    movb 0(%r12, %r14), %al
    testb %al, %al
    jnz .jnz_label
